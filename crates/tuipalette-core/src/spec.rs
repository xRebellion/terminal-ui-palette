//! The declarative half: a document describes what is on screen.
//!
//! Every field pins its wire name with `rename`, and every struct refuses
//! unknown fields. Both matter for the same reason: RON's type names *are* the
//! file format (docs/PLAN.md, Risks), so a Rust refactor must break the
//! compiler, not somebody's spec.
//!
//! One deliberate deviation from the spike: a layout node is a single struct
//! with `dir`/`children`/`pane` rather than an enum of variants. The enum
//! would read a little better, but serde cannot deny unknown fields inside a
//! variant, and a typo'd `childrens:` would then silently produce an empty
//! split. The struct shape trades one word of syntax for that whole class of
//! error.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ratatui_core::style::Color;
use serde::Deserialize;

use crate::action::Action;
use crate::error::{Location, SpecError};
use crate::layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Ron,
    Json,
}

impl Format {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ron" => Some(Self::Ron),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "ron" => Some(Self::Ron),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpec {
    #[serde(rename = "window")]
    pub window: Window,
    #[serde(rename = "theme", default)]
    pub theme: Theme,
    #[serde(rename = "layout")]
    pub layout: Node,
    #[serde(rename = "panes", default)]
    pub panes: Vec<Pane>,
    #[serde(rename = "keys", default)]
    pub keys: BTreeMap<String, Option<Action>>,
}

/// RON needs one extension to read the way the docs say it does: without
/// `IMPLICIT_SOME` every `Option` field would have to be written `Some(...)`,
/// and `dir: Vertical` would be a parse error.
fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

impl UiSpec {
    /// Read, parse, then validate. `format` overrides the file extension.
    pub fn load(path: &Path, format: Option<Format>) -> Result<Self, SpecError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| SpecError::plain(format!("{}: {e}", path.display())))?;
        let format = format
            .or_else(|| Format::from_path(path))
            .unwrap_or(Format::Ron);
        Self::parse(&text, format)
    }

    pub fn parse(text: &str, format: Format) -> Result<Self, SpecError> {
        let spec: UiSpec = match format {
            Format::Ron => ron_options().from_str(text).map_err(|e| {
                SpecError::at_location(
                    Location {
                        line: e.span.start.line,
                        column: e.span.start.col,
                    },
                    e.code.to_string(),
                )
            })?,
            Format::Json => serde_json::from_str(text).map_err(|e| {
                SpecError::at_location(
                    Location {
                        line: e.line(),
                        column: e.column(),
                    },
                    e.to_string(),
                )
            })?,
        };
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<(), SpecError> {
        for (field, value) in [
            ("accent", &self.theme.accent),
            ("selected_bg", &self.theme.selected_bg),
            ("text", &self.theme.text),
        ] {
            if parse_color(value).is_none() {
                return Err(SpecError::at_path(
                    format!("theme.{field}"),
                    format!("expected a hex colour such as #89b4fa, found `{value}`"),
                ));
            }
        }

        let mut ids = BTreeSet::new();
        for (i, pane) in self.panes.iter().enumerate() {
            if pane.id.trim().is_empty() {
                return Err(SpecError::at_path(
                    format!("panes[{i}].id"),
                    "a pane id must not be empty",
                ));
            }
            if !ids.insert(pane.id.clone()) {
                return Err(SpecError::at_path(
                    format!("panes[{i}].id"),
                    format!("duplicate pane id `{}`", pane.id),
                ));
            }
            if pane.icon_width == Some(0) {
                return Err(SpecError::at_path(
                    format!("panes[{i}].icon_width"),
                    "an icon column is at least 1 cell wide; omit `icon_width` for no column",
                ));
            }
        }

        let mut drawn = BTreeSet::new();
        layout::validate(&self.layout, "layout", &ids, &mut drawn)?;

        // A key string that is not a key would silently bind nothing, so it is a
        // spec error with its path (D8).
        crate::keymap::Keymap::from_spec(self)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    #[serde(rename = "title", default)]
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Theme {
    #[serde(rename = "accent")]
    pub accent: String,
    #[serde(rename = "selected_bg")]
    pub selected_bg: String,
    #[serde(rename = "text")]
    pub text: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: "#89b4fa".to_owned(),
            selected_bg: "#313244".to_owned(),
            text: "#cdd6f4".to_owned(),
        }
    }
}

impl Theme {
    pub fn accent_color(&self) -> Color {
        parse_color(&self.accent).unwrap_or(Color::Reset)
    }
    pub fn selected_color(&self) -> Color {
        parse_color(&self.selected_bg).unwrap_or(Color::Reset)
    }
    pub fn text_color(&self) -> Color {
        parse_color(&self.text).unwrap_or(Color::Reset)
    }
}

/// `#rrggbb`, nothing else. A spec is validated, not guessed at.
pub fn parse_color(spec: &str) -> Option<Color> {
    let hex = spec.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(Color::Rgb(pair(0)?, pair(2)?, pair(4)?))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pane {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "kind", default)]
    pub kind: Kind,
    #[serde(rename = "title", default)]
    pub title: String,
    #[serde(rename = "items", default)]
    pub items: Vec<String>,
    /// Width of the icon column in cells; `None` means the pane has none.
    #[serde(rename = "icon_width", default)]
    pub icon_width: Option<u8>,
    #[serde(rename = "keys", default)]
    pub keys: BTreeMap<String, Option<Action>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub enum Kind {
    #[serde(rename = "Text")]
    #[default]
    Text,
    #[serde(rename = "Input")]
    Input,
    #[serde(rename = "Menu")]
    Menu,
}

/// One node of the layout tree: either a leaf that names a pane, or a split
/// with children.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(rename = "dir", default)]
    pub dir: Option<Axis>,
    #[serde(rename = "sizes", default)]
    pub sizes: Vec<String>,
    #[serde(rename = "spacing", default)]
    pub spacing: u16,
    #[serde(rename = "children", default)]
    pub children: Vec<Node>,
    #[serde(rename = "pane", default)]
    pub pane: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Axis {
    #[serde(rename = "Vertical")]
    Vertical,
    #[serde(rename = "Horizontal")]
    Horizontal,
}
