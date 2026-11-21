//! The router: one focus ring, one key map, one dispatch order.
//!
//! Order is fixed and small — global actions first, then the focused pane, then
//! every other component once, stopping at the first handler — so a component
//! never has to know what else is on screen (docs/PLAN.md, 04).

use std::collections::BTreeMap;

use crate::action::Action;
use crate::component::{Component, ImageRequest, Paint, Reaction};
use crate::error::SpecError;
use crate::key::{Key, KeyCode};
use crate::keymap::Keymap;
use crate::layout;
use crate::pointer::{Mouse, MouseKind};
use crate::spec::UiSpec;
use crate::{Buffer, Rect};

/// How many times an `Emit` may re-enter dispatch before the router stops, so a
/// component that emits the action it just handled cannot spin forever.
const MAX_EMIT_DEPTH: usize = 8;

/// What the app should do after an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The action was absorbed; keep running.
    Continue,
    /// A global bind asked to stop.
    Quit,
    /// A component asked the app to run `command`; `value` is what it would hand
    /// back if the app treats this as an accept.
    Invoke {
        command: String,
        value: Option<String>,
    },
}

pub struct Session {
    spec: UiSpec,
    components: Vec<Box<dyn Component>>,
    by_id: BTreeMap<String, usize>,
    /// Component indices in draw order, focusable only: the focus ring.
    ring: Vec<usize>,
    /// Position in `ring`, not a component index.
    slot: Option<usize>,
    keymap: Keymap,
}

impl Session {
    /// Build the router around the spec and the components the caller made for
    /// its panes. Components may arrive in any order; the ring follows the
    /// layout, because that is the order the user sees.
    pub fn new(spec: UiSpec, components: Vec<Box<dyn Component>>) -> Result<Self, SpecError> {
        spec.validate()?;
        let keymap = Keymap::from_spec(&spec)?;
        let mut by_id = BTreeMap::new();
        for (i, component) in components.iter().enumerate() {
            if by_id.insert(component.id().to_owned(), i).is_some() {
                return Err(SpecError::plain(format!(
                    "two components claim pane `{}`",
                    component.id()
                )));
            }
        }
        let ring: Vec<usize> = layout::pane_order(&spec.layout)
            .into_iter()
            .filter_map(|id| by_id.get(id).copied())
            .filter(|&i| components[i].focusable())
            .collect();
        let slot = (!ring.is_empty()).then_some(0);
        Ok(Self {
            spec,
            components,
            by_id,
            ring,
            slot,
            keymap,
        })
    }

    /// Build every pane's built-in component from its `kind`.
    pub fn from_spec(spec: UiSpec) -> Result<Self, SpecError> {
        let components: Vec<Box<dyn Component>> = spec
            .panes
            .iter()
            .map(crate::components::from_pane)
            .collect();
        Self::new(spec, components)
    }

    pub fn spec(&self) -> &UiSpec {
        &self.spec
    }

    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// What one pane would hand back, whether or not it holds focus.
    pub fn value_of(&self, id: &str) -> Option<String> {
        let index = *self.by_id.get(id)?;
        self.components[index].value()
    }

    /// A pane's component by type.
    pub fn pane<T: Component>(&self, id: &str) -> Option<&T> {
        let index = *self.by_id.get(id)?;
        let any: &dyn std::any::Any = &*self.components[index];
        any.downcast_ref::<T>()
    }

    /// A pane's component by type — how the picker reaches the menu it fills.
    /// `None` when the id is unknown or the component is something else.
    pub fn pane_mut<T: Component>(&mut self, id: &str) -> Option<&mut T> {
        let index = *self.by_id.get(id)?;
        let any: &mut dyn std::any::Any = &mut *self.components[index];
        any.downcast_mut::<T>()
    }

    /// The focused pane's id, or `None` when nothing on screen takes focus.
    pub fn focus(&self) -> Option<&str> {
        let i = self.ring.get(self.slot?).copied()?;
        Some(self.components[i].id())
    }

    /// What the focused component would hand back.
    pub fn value(&self) -> Option<String> {
        let i = self.ring.get(self.slot?).copied()?;
        self.components[i].value()
    }

    /// Resolve a key for the focused pane, then either dispatch its bind or hand
    /// the key to the focused component — typing works without a bind per
    /// letter, and so does editing, which is why Delete is here too.
    pub fn on_key(&mut self, key: Key) -> Outcome {
        let pane = self.focus().unwrap_or_default().to_owned();
        if let Some(action) = self.keymap.resolve(&pane, key).cloned() {
            return self.dispatch(action);
        }
        match key.code {
            KeyCode::Char(c) if !key.mods.ctrl && !key.mods.alt => self.dispatch(Action::Input(c)),
            KeyCode::Backspace => self.dispatch(Action::Backspace),
            KeyCode::Delete => self.dispatch(Action::Delete),
            _ => Outcome::Continue,
        }
    }

    pub fn dispatch(&mut self, action: Action) -> Outcome {
        self.fire(action, 0)
    }

    fn fire(&mut self, action: Action, depth: usize) -> Outcome {
        if depth > MAX_EMIT_DEPTH {
            return Outcome::Continue;
        }
        match &action {
            Action::Quit => return Outcome::Quit,
            Action::FocusNext => {
                self.cycle(1);
                return Outcome::Continue;
            }
            Action::FocusPrev => {
                self.cycle(-1);
                return Outcome::Continue;
            }
            _ => {}
        }

        let first = self.slot.and_then(|slot| self.ring.get(slot)).copied();
        for i in self.order_from(first) {
            match self.components[i].update(&action) {
                Reaction::Ignored => {}
                Reaction::Handled => return Outcome::Continue,
                Reaction::Emit(next) => {
                    if let Action::Invoke(command) = next {
                        let value = self.components[i].value();
                        return Outcome::Invoke { command, value };
                    }
                    return self.fire(next, depth + 1);
                }
            }
        }
        Outcome::Continue
    }

    /// The focused component first, then everything else in declaration order —
    /// bubbling reaches a component that does not hold focus but does know the
    /// action.
    fn order_from(&self, first: Option<usize>) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.components.len());
        if let Some(first) = first {
            order.push(first);
        }
        order.extend((0..self.components.len()).filter(|i| Some(*i) != first));
        order
    }

    fn cycle(&mut self, step: isize) {
        if self.ring.is_empty() {
            return;
        }
        let n = self.ring.len() as isize;
        let mut slot = self.slot.unwrap_or(0) as isize;
        slot = (slot + step).rem_euclid(n);
        self.slot = Some(slot as usize);
    }

    /// Move focus to a pane, if it is one the ring can land on.
    pub fn focus_on(&mut self, id: &str) -> bool {
        let Some(slot) = self
            .ring
            .iter()
            .position(|&i| self.components[i].id() == id)
        else {
            return false;
        };
        self.slot = Some(slot);
        true
    }

    /// The rect every pane is drawn in for a frame of `area` — the one source
    /// both drawing and hit-testing read, so a click lands where the eye sees.
    fn rects(&self, area: Rect) -> Vec<(String, Rect)> {
        let area = if self.spec.window.frame {
            crate::ui::inner(area)
        } else {
            area
        };
        let pad = self.spec.window.padding;
        let area = Rect::new(
            area.x.saturating_add(pad.x),
            area.y.saturating_add(pad.y),
            area.width.saturating_sub(pad.x.saturating_mul(2)),
            area.height.saturating_sub(pad.y.saturating_mul(2)),
        );
        // A validated spec cannot fail to resolve, and the spec is validated in
        // 'new', so nothing is drawn rather than panicking if that ever changes.
        layout::resolve(&self.spec.layout, area).unwrap_or_default()
    }

    /// A pointer event over a frame of `area`. A click first moves focus to
    /// the pane under it (when that pane can hold focus); then every pane is
    /// offered the event with its own rect, so the one under the pointer can
    /// act and the others can drop a hover. An `Invoke` from a pane is an
    /// accept, exactly as from a key.
    ///
    /// A document that did not ask for the mouse ignores it here too, not only
    /// by leaving capture off: a terminal can still deliver a mouse sequence
    /// (another program enabled reporting, or bytes arrive on the tty), and
    /// the opt-in has to mean the picker never acts on one.
    pub fn on_mouse(&mut self, area: Rect, mouse: Mouse) -> Outcome {
        if !self.spec.window.mouse {
            return Outcome::Continue;
        }
        let rects = self.rects(area);
        if mouse.kind == MouseKind::Click {
            let hit = rects
                .iter()
                .find(|(_, rect)| rect.contains(mouse.position()))
                .map(|(id, _)| id.clone());
            if let Some(id) = hit {
                self.focus_on(&id);
            }
        }
        let mut outcome = Outcome::Continue;
        for (id, rect) in rects {
            let Some(&i) = self.by_id.get(&id) else {
                continue;
            };
            match self.components[i].on_mouse(&mouse, rect) {
                Reaction::Ignored | Reaction::Handled => {}
                Reaction::Emit(Action::Invoke(command)) => {
                    let value = self.components[i].value();
                    outcome = Outcome::Invoke { command, value };
                }
                Reaction::Emit(next) => {
                    let next = self.dispatch(next);
                    if next != Outcome::Continue {
                        outcome = next;
                    }
                }
            }
        }
        outcome
    }

    /// The pictures this frame of `area` asks for, one per pane that wants
    /// one, in the rects `render` drew their fallbacks in. A host with
    /// graphics draws these after the cells; one without ignores them.
    pub fn images(&self, area: Rect) -> Vec<ImageRequest> {
        self.rects(area)
            .into_iter()
            .filter_map(|(id, rect)| {
                let &i = self.by_id.get(&id)?;
                self.components[i].image(rect)
            })
            .collect()
    }

    /// Paint every pane in the rect the layout gave it.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // One frame around the whole picker, if the document asks for one:
        // the panes inside then draw into its interior, which is how a
        // single-boxed layout with dividers inside is written.
        if self.spec.window.frame {
            // The window's title sits on its frame, the way a framed pane's does.
            crate::ui::frame(buf, area, &self.spec.window.title, false, &self.spec.theme);
        }
        let rects = self.rects(area);
        let focused = self.focus().map(str::to_owned);
        for (id, rect) in rects {
            let Some(&i) = self.by_id.get(&id) else {
                continue;
            };
            let paint = Paint {
                area: rect,
                focused: focused.as_deref() == Some(id.as_str()),
                theme: &self.spec.theme,
            };
            self.components[i].render(&paint, buf);
        }
    }
}
