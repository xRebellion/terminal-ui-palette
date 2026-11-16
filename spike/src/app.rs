//! The wiring: focus ring, action dispatch, draw pass — and the tests that
//! carry this spike's three claims.

use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::action::{Action, KeyMaps};
use crate::component::{from_pane, Component, Reaction};
use crate::spec::{resolve, UiSpec};

pub struct App {
    pub spec: UiSpec,
    pub panes: Vec<Box<dyn Component>>,
    pub keymaps: KeyMaps,
    pub focus: usize,
    pub quit: bool,
    /// Last command a component asked the app to run.
    pub last_command: Option<String>,
    /// What the picker hands back on accept.
    pub result: Option<String>,
}

impl App {
    pub fn new(spec: UiSpec) -> Self {
        let panes: Vec<Box<dyn Component>> = spec.panes.iter().map(from_pane).collect();
        let pane_keys: Vec<(String, std::collections::BTreeMap<String, Option<Action>>)> =
            spec.panes
                .iter()
                .map(|p| (p.id.clone(), p.keys.clone()))
                .collect();
        let keymaps = KeyMaps::build(&spec.keys, &pane_keys);
        let focus = panes.iter().position(|p| p.focusable()).unwrap_or(0);
        Self {
            spec,
            panes,
            keymaps,
            focus,
            quit: false,
            last_command: None,
            result: None,
        }
    }

    pub fn take_result(&mut self) -> Option<String> {
        self.result.take()
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        let pane = self
            .panes
            .get(self.focus)
            .map(|p| p.id().to_string())
            .unwrap_or_default();
        if let Some(action) = self.keymaps.resolve(&pane, key) {
            self.dispatch(action);
        }
    }

    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.quit = true;
                return;
            }
            Action::FocusNext => return self.cycle(1),
            Action::FocusPrev => return self.cycle(-1),
            _ => {}
        }

        if let Some(i) = self.focused_index() {
            match self.panes[i].update(&action) {
                Reaction::Ignored => {}
                Reaction::Handled => return,
                Reaction::Emit(next) => return self.emit(next, i),
            }
        }
        for i in 0..self.panes.len() {
            if Some(i) == self.focused_index() {
                continue;
            }
            match self.panes[i].update(&action) {
                Reaction::Ignored => {}
                Reaction::Handled => return,
                Reaction::Emit(next) => return self.emit(next, i),
            }
        }
    }

    fn emit(&mut self, action: Action, from: usize) {
        if let Action::Invoke(cmd) = action {
            self.last_command = Some(cmd);
            self.result = self.panes[from].value();
            self.quit = true;
        }
    }

    fn focused_index(&self) -> Option<usize> {
        self.panes
            .get(self.focus)
            .filter(|p| p.focusable())
            .map(|_| self.focus)
    }

    fn cycle(&mut self, step: isize) {
        let n = self.panes.len() as isize;
        if n == 0 {
            return;
        }
        let mut i = self.focus as isize;
        for _ in 0..n {
            i = (i + step).rem_euclid(n);
            if self.panes[i as usize].focusable() {
                self.focus = i as usize;
                return;
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let theme = &self.spec.theme;
        let root = frame.area();
        let block = Block::bordered()
            .title(format!(" {} ", self.spec.window.title))
            .border_style(Style::default().fg(theme.accent_color()));
        let inner = block.inner(root);
        frame.render_widget(block, root);

        let mut rects = Vec::new();
        resolve(&self.spec.layout, inner, &mut rects);
        let focused_id = self
            .panes
            .get(self.focus)
            .map(|p| p.id().to_string())
            .unwrap_or_default();
        let buf = frame.buffer_mut();
        for (id, rect) in rects {
            if let Some(pane) = self.panes.iter().find(|p| p.id() == id) {
                pane.render(rect, buf, id == focused_id, theme);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use ratatui::Terminal;

    fn app() -> App {
        App::new(UiSpec::parse(include_str!("../spec.ron")).expect("spec.ron must parse"))
    }

    /// Claim 1: the spec's rects are the rects the panes actually draw on.
    #[test]
    fn panes_land_in_their_rects() {
        let app = app();
        let screen = Rect::new(0, 0, 60, 14);
        let mut term = Terminal::new(TestBackend::new(screen.width, screen.height)).unwrap();
        term.draw(|frame| app.draw(frame)).unwrap();

        let inner = Block::bordered().inner(screen);
        let mut rects = Vec::new();
        resolve(&app.spec.layout, inner, &mut rects);
        assert_eq!(rects.len(), 3, "three panes in the spec");

        let buf = term.backend().buffer();
        for (id, rect) in &rects {
            let cell = buf
                .cell((rect.x, rect.y))
                .unwrap_or_else(|| panic!("pane {id} rect outside the buffer"));
            assert_eq!(
                cell.symbol(),
                "┌",
                "pane {id} should begin on a border corner at {rect:?}"
            );
        }

        let (_, results) = rects.iter().find(|(id, _)| id == "results").unwrap();
        let row: String = (results.x..results.x + results.width)
            .map(|x| {
                buf.cell((x, results.y + 1))
                    .map(|c| c.symbol().to_string())
                    .unwrap_or_default()
            })
            .collect();
        assert!(row.contains("firefox"), "first item row was {row:?}");
    }

    /// Claim 2: down, down, enter traverses the Component trait and yields the
    /// third item — including the bubble from the focused input to the menu.
    #[test]
    fn down_down_enter_yields_the_third_item() {
        let mut app = app();
        app.dispatch(Action::Down);
        app.dispatch(Action::Down);
        app.dispatch(Action::Accept);
        assert_eq!(app.last_command.as_deref(), Some("run:herdr"));
        assert_eq!(app.result.as_deref(), Some("herdr"));
        assert!(app.quit);
    }

    /// Claim 3 (D8): a per-pane entry wins, and an explicit `None` unsets an
    /// inherited global bind instead of falling through to it.
    #[test]
    fn per_pane_override_wins_and_explicit_none_unsets() {
        let app = app();
        let ctrl = |c: char| {
            ratatui::crossterm::event::KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
        };
        assert_eq!(app.keymaps.resolve("results", ctrl('j')), None);
        assert_eq!(app.keymaps.resolve("query", ctrl('j')), Some(Action::Down));
        assert_eq!(app.keymaps.resolve("results", ctrl('n')), Some(Action::Down));
    }

    /// Claim 4: a broken spec reports where, not `None`.
    #[test]
    fn malformed_spec_reports_a_position() {
        let syntax = UiSpec::parse("(window: (title: \"x\"), layout: Vertical(children: [Pane(\"a\"))")
            .unwrap_err();
        assert!(syntax.contains("line"), "{syntax}");
        let schema =
            UiSpec::parse("(window: (title: \"x\"), layout: Pane(\"a\"), bogus: true)").unwrap_err();
        assert!(schema.contains("line"), "{schema}");
    }
}
