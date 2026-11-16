//! The picker: the one thing that is more than the sum of its panes.
//!
//! It owns what streams in, what the user typed, and the rows that survive —
//! then hands those rows to the menu pane every time either changes. Everything
//! else (focus, keys, layout, drawing) is the session's, unchanged.

use std::sync::mpsc::{Receiver, TryRecvError};

use crate::action::Action;
use crate::components::{Item, Menu, Row};
use crate::error::SpecError;
use crate::key::Key;
use crate::matcher::{default_matcher, Matcher};
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
}

impl Picker {
    pub fn new(spec: UiSpec) -> Result<Self, SpecError> {
        let input = first_of_kind(&spec, Kind::Input);
        let menu = first_of_kind(&spec, Kind::Menu);
        let session = Session::from_spec(spec)?;
        Ok(Self {
            session,
            input,
            menu,
            matcher: default_matcher(),
            items: Vec::new(),
            query: String::new(),
            sorting: true,
            accept_query: false,
        })
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
        outcome
    }

    pub fn dispatch(&mut self, action: Action) -> Outcome {
        let before = self.current_query();
        let outcome = self.session.dispatch(action);
        self.sync_query(before);
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
    /// menu. Score decides the order; equal scores keep arrival order.
    fn refilter(&mut self) {
        let Some(menu) = self.menu.clone() else {
            return;
        };
        self.matcher.set_query(&self.query);
        let mut scored: Vec<(u32, usize, Row)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let hit = self.matcher.score(&item.display)?;
                Some((
                    hit.score,
                    index,
                    Row {
                        item: item.clone(),
                        positions: hit.positions,
                    },
                ))
            })
            .collect();
        if self.sorting {
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        }
        let rows = scored.into_iter().map(|(_, _, row)| row).collect();
        if let Some(menu) = self.session.pane_mut::<Menu>(&menu) {
            menu.set_rows(rows);
        }
    }
}

fn first_of_kind(spec: &UiSpec, kind: Kind) -> Option<String> {
    spec.panes
        .iter()
        .find(|pane| pane.kind == kind)
        .map(|pane| pane.id.clone())
}
