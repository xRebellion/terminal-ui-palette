//! The picker: the one thing that is more than the sum of its panes.
//!
//! It owns what streams in, what the user typed, and the rows that survive —
//! then hands those rows to the menu pane every time either changes. Everything
//! else (focus, keys, layout, drawing) is the session's, unchanged.

use std::sync::mpsc::{Receiver, TryRecvError};

use crate::action::Action;
use crate::components::{CounterPane, DetailPane, Item, Menu, Row};
use crate::error::SpecError;
use crate::key::Key;
use crate::matcher::{default_matcher, Matcher};
use crate::rank::{self, Tier};
use crate::session::{Outcome, Session};
use crate::spec::{Kind, UiSpec};
use crate::{Buffer, Rect};

pub struct Picker {
    session: Session,
    /// The pane that holds the query, if the document has one.
    input: Option<String>,
    /// The pane the rows are written into, if the document has one.
    menu: Option<String>,
    matcher: Box<dyn Matcher>,
    /// Everything that has arrived, unfiltered.
    items: Vec<Item>,
    query: String,
    /// Keep the provider's order instead of ranking (rofi's `-no-sort`).
    sorting: bool,
    /// Accept the typed query when nothing matches (rofi's custom input).
    accept_query: bool,
    /// Pairs of (menu pane, detail pane): the preview follows the menu.
    detail_links: Vec<(String, String)>,
    /// The panes that show `shown/total`. There may be more than one, and none
    /// is fine — a document without a counter is the document it always was.
    counters: Vec<String>,
}

impl Picker {
    pub fn new(spec: UiSpec) -> Result<Self, SpecError> {
        let input = first_of_kind(&spec, Kind::Input);
        let menu = first_of_kind(&spec, Kind::Menu);
        let detail_links = detail_links(&spec);
        let counters = ids_of_kind(&spec, Kind::Counter);
        let session = Session::from_spec(spec)?;
        let mut picker = Self {
            session,
            input,
            menu,
            matcher: default_matcher(),
            items: Vec::new(),
            query: String::new(),
            sorting: true,
            accept_query: false,
            detail_links,
            counters,
        };
        picker.sync_detail();
        picker.sync_counters();
        Ok(picker)
    }

    /// Keep the order items arrived in. Filtering and highlighting still
    /// happen; only the ranking is skipped, which is what a caller whose rows
    /// already carry meaning asks for (`rofi -no-sort`).
    pub fn no_sort(mut self, yes: bool) -> Self {
        self.sorting = !yes;
        self.refilter();
        self
    }

    /// Answer `Accept` with the typed query when no row is selected. This is
    /// `rofi -dmenu`'s default behaviour, and the only way a prompt can take
    /// free text as its answer.
    pub fn accept_query(mut self, yes: bool) -> Self {
        self.accept_query = yes;
        self
    }

    /// Use a different scorer — the tests use this to pin ordering.
    pub fn with_matcher(mut self, matcher: Box<dyn Matcher>) -> Self {
        self.matcher = matcher;
        self.refilter();
        self
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// Every item received, in arrival order.
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// The rows the menu is showing, best match first.
    pub fn rows(&self) -> Vec<Row> {
        self.menu
            .as_deref()
            .and_then(|id| self.session.pane::<Menu>(id))
            .map(|menu| menu.rows().to_vec())
            .unwrap_or_default()
    }

    /// Append items as they stream in.
    pub fn push(&mut self, items: impl IntoIterator<Item = Item>) {
        self.items.extend(items);
        self.refilter();
    }

    /// Take whatever the source has produced since the last frame. Returns
    /// whether anything arrived, which is also whether a redraw is due.
    pub fn drain(&mut self, source: &Receiver<Item>) -> bool {
        let mut arrived = false;
        loop {
            match source.try_recv() {
                Ok(item) => {
                    self.items.push(item);
                    arrived = true;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if arrived {
            self.refilter();
        }
        arrived
    }

    pub fn on_key(&mut self, key: Key) -> Outcome {
        // Free text as the answer: only when nothing is selected, and only
        // when the caller asked for it — otherwise Enter on an empty list
        // stays silent, which is what a fixed-row menu wants.
        if self.accept_query && self.rows().is_empty() {
            let focused = self.session.focus().unwrap_or_default().to_owned();
            if matches!(
                self.session.keymap().resolve(&focused, key),
                Some(Action::Accept)
            ) {
                let query = self.query.clone();
                return Outcome::Invoke { command: query.clone(), value: Some(query) };
            }
        }
        // A document with no query pane still types: the query lives here and
        // the rows still narrow. A bind always wins over a character.
        if self.input.is_none() {
            let focused = self.session.focus().unwrap_or_default().to_owned();
            if self.session.keymap().resolve(&focused, key).is_none() {
                match key.code {
                    crate::key::KeyCode::Char(c) if !key.mods.ctrl && !key.mods.alt => {
                        self.query.push(c);
                        self.refilter();
                        return Outcome::Continue;
                    }
                    crate::key::KeyCode::Backspace | crate::key::KeyCode::Delete => {
                        self.query.pop();
                        self.refilter();
                        return Outcome::Continue;
                    }
                    _ => {}
                }
            }
        }
        let before = self.current_query();
        let outcome = self.session.on_key(key);
        self.sync_query(before);
        self.sync_detail();
        outcome
    }

    pub fn dispatch(&mut self, action: Action) -> Outcome {
        let before = self.current_query();
        let outcome = self.session.dispatch(action);
        self.sync_query(before);
        self.sync_detail();
        outcome
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        self.session.render(area, buf);
    }

    fn current_query(&self) -> String {
        self.input
            .as_deref()
            .and_then(|id| self.session.value_of(id))
            .unwrap_or_default()
    }

    fn sync_query(&mut self, before: String) {
        let after = self.current_query();
        if after != before {
            self.query = after;
            self.refilter();
        }
    }

    /// Re-score everything against the query and hand the surviving rows to the
    /// menu. The match tier decides the order, then — in the fuzzy tier only —
    /// the scorer's number, then arrival order, which is the provider's own
    /// ranking (usage, for the launchers). See `rank.rs` and D20.
    fn refilter(&mut self) {
        let Some(menu) = self.menu.clone() else {
            self.sync_counters();
            return;
        };
        self.matcher.set_query(&self.query);
        let query = self.query.trim();
        let mut scored: Vec<(Tier, u32, usize, Row)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                // A literal hit is ranked by its tier alone and highlighted
                // where it was found, so the colour agrees with the reason.
                if let Some((tier, positions)) = rank::literal(query, &item.display) {
                    let row = Row { item: item.clone(), positions };
                    return Some((tier, 0, index, row));
                }
                let (tier, score, positions) = match self.matcher.score(&item.display) {
                    Some(hit) => (Tier::Fuzzy, hit.score, hit.positions),
                    None => {
                        // The row is found by words it does not show (its value,
                        // or the provider's search text). It is a real hit, but
                        // a shallower one than any display match, and there is
                        // nothing to highlight: positions index the display.
                        let value = (item.value != item.display)
                            .then(|| self.matcher.score(&item.value))
                            .flatten();
                        let found = value.or_else(|| {
                            item.search
                                .as_deref()
                                .and_then(|search| self.matcher.score(search))
                        });
                        found?;
                        (Tier::Hidden, 0, Vec::new())
                    }
                };
                Some((
                    tier,
                    score,
                    index,
                    Row {
                        item: item.clone(),
                        positions,
                    },
                ))
            })
            .collect();
        if self.sorting {
            scored.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        }
        let rows = scored.into_iter().map(|(_, _, _, row)| row).collect();
        if let Some(menu) = self.session.pane_mut::<Menu>(&menu) {
            menu.set_rows(rows);
        }
        self.sync_detail();
        self.sync_counters();
    }

    /// Copy the selected row's detail into every detail pane that follows
    /// it. Called after anything that can move the selection or change the
    /// rows, so a preview is never one keystroke behind.
    fn sync_detail(&mut self) {
        for (menu, detail) in self.detail_links.clone() {
            let text = self
                .session
                .pane::<Menu>(&menu)
                .and_then(|menu| menu.selected())
                .and_then(|item| item.detail.clone());
            if let Some(pane) = self.session.pane_mut::<DetailPane>(&detail) {
                pane.set_text(text);
            }
        }
    }

    /// Write both numbers into every counter pane. The picker is the only
    /// thing that holds them: what arrived lives here, what survived lives in
    /// the menu, and the total is the larger of the two — a document whose menu
    /// declares its rows statically has no streamed items at all, and its
    /// counter still has to say how many rows there are (docs/PLAN.md, 23).
    fn sync_counters(&mut self) {
        let shown = self
            .menu
            .as_deref()
            .and_then(|id| self.session.pane::<Menu>(id))
            .map(|menu| menu.rows().len())
            .unwrap_or(0);
        let total = self.items.len().max(shown);
        for id in self.counters.clone() {
            if let Some(pane) = self.session.pane_mut::<CounterPane>(&id) {
                pane.set_counts(shown, total);
            }
        }
    }
}

/// Every detail pane in the document, paired with the menu it follows —
/// the one `from` names, or the first menu there is.
fn detail_links(spec: &UiSpec) -> Vec<(String, String)> {
    let first_menu = first_of_kind(spec, Kind::Menu);
    spec.panes
        .iter()
        .filter(|pane| pane.kind == Kind::Detail)
        .filter_map(|pane| {
            let menu = pane.from.clone().or_else(|| first_menu.clone())?;
            Some((menu, pane.id.clone()))
        })
        .collect()
}

fn first_of_kind(spec: &UiSpec, kind: Kind) -> Option<String> {
    spec.panes
        .iter()
        .find(|pane| pane.kind == kind)
        .map(|pane| pane.id.clone())
}

/// Every pane of a kind, in declaration order.
fn ids_of_kind(spec: &UiSpec, kind: Kind) -> Vec<String> {
    spec.panes
        .iter()
        .filter(|pane| pane.kind == kind)
        .map(|pane| pane.id.clone())
        .collect()
}
