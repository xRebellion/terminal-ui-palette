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
    /// How the host should present this picker — a hint for the adapters; the
    /// library itself never places a window (docs/PLAN.md, D3).
    #[serde(rename = "host", default)]
    pub host: Host,
}

/// What an adapter needs to know about the window it should make.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    /// Width in cells.
    #[serde(rename = "width", default)]
    pub width: Option<u16>,
    /// Height in lines.
    #[serde(rename = "height", default)]
    pub height: Option<u16>,
    /// Window class the floating adapter asks its terminal for, so a compositor
    /// rule can match it.
    #[serde(rename = "class", default)]
    pub class: Option<String>,
}

/// RON needs one extension to read the way the docs say it does: without
/// `IMPLICIT_SOME` every `Option` field would have to be written `Some(...)`,
/// and `dir: Vertical` would be a parse error.
/// Parse a theme file. Same options as a whole document, so `#rrggbb` reads
/// the same in both.
pub fn parse_theme(text: &str) -> Result<Theme, SpecError> {
    ron_options().from_str(text).map_err(|e| {
        SpecError::at_location(
            Location {
                line: e.span.start.line,
                column: e.span.start.col,
            },
            e.code.to_string(),
        )
    })
}

fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

impl UiSpec {
    /// `WxH` when the document says, for the floating adapter's script.
    pub fn geometry(&self) -> Option<String> {
        match (self.host.width, self.host.height) {
            (Some(width), Some(height)) => Some(format!("{width}x{height}")),
            _ => None,
        }
    }

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
            ("dim", &self.theme.dim),
            ("scrollbar", &self.theme.scrollbar),
            ("scrollbar_bg", &self.theme.scrollbar_bg),
            ("hover_bg", &self.theme.hover_bg),
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
            if let Some(from) = &pane.from {
                match self.panes.iter().find(|other| &other.id == from) {
                    Some(other) if other.kind == Kind::Menu => {}
                    Some(_) => {
                        return Err(SpecError::at_path(
                            format!("panes[{i}].from"),
                            format!("`{from}` is not a menu; a detail pane follows a Menu"),
                        ));
                    }
                    None => {
                        return Err(SpecError::at_path(
                            format!("panes[{i}].from"),
                            format!("no pane has the id `{from}`"),
                        ));
                    }
                }
            }
            if pane.icon_width == Some(0) {
                return Err(SpecError::at_path(
                    format!("panes[{i}].icon_width"),
                    "an icon column is at least 1 cell wide; omit `icon_width` for no column",
                ));
            }
            if pane.scrollbar && pane.kind != Kind::Menu {
                return Err(SpecError::at_path(
                    format!("panes[{i}].scrollbar"),
                    "only a Menu has a position in it; `scrollbar` belongs to a list",
                ));
            }
            if let Some(style) = pane.scrollbar_style {
                // A style with nothing to style is a document that thinks it
                // draws a bar and does not, so both halves are named here.
                if pane.kind != Kind::Menu {
                    return Err(SpecError::at_path(
                        format!("panes[{i}].scrollbar_style"),
                        "only a Menu has a position in it; `scrollbar_style` belongs to a list",
                    ));
                }
                if !pane.scrollbar {
                    return Err(SpecError::at_path(
                        format!("panes[{i}].scrollbar_style"),
                        format!(
                            "`{}` is the style of a bar this pane does not draw; add `scrollbar: true`",
                            style.name()
                        ),
                    ));
                }
            }
            if pane.kind != Kind::Counter {
                // Name the field the document actually wrote, so the error
                // points at the line to delete.
                let stray = if pane.format.is_some() {
                    Some("format")
                } else if pane.align.is_some() {
                    Some("align")
                } else {
                    None
                };
                if let Some(field) = stray {
                    return Err(SpecError::at_path(
                        format!("panes[{i}].{field}"),
                        "only a Counter writes numbers; `format` and `align` belong to one",
                    ));
                }
            }
            if let Some(written) = &pane.format
                && let Some(bad) = unfillable(written)
            {
                return Err(SpecError::at_path(
                    format!("panes[{i}].format"),
                    format!(
                        "`{bad}` cannot be filled in — a counter knows `{{shown}}` and `{{total}}`"
                    ),
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

/// The first thing in a `format` a counter cannot fill in: an unknown
/// `{placeholder}` or a brace that is not part of one. A document that asks for
/// `{rows}` is a document whose counter would print the word instead, which is
/// the kind of typo a validated spec exists to catch.
///
/// The scan runs over the string as written — valid placeholders are stepped
/// over, never deleted first — so the quoted fragment is always a piece of
/// the document's own text.
fn unfillable(format: &str) -> Option<String> {
    let mut from = 0;
    while let Some(offset) = format[from..].find(['{', '}']) {
        let at = from + offset;
        let tail = &format[at..];
        if tail.starts_with('}') {
            return Some("}".to_owned());
        }
        if let Some(known) = ["{shown}", "{total}"].iter().find(|p| tail.starts_with(**p)) {
            from = at + known.len();
            continue;
        }
        // Up to the brace that closes this one, or up to (not including) the
        // next opening brace when a second one starts first.
        let end = match tail[1..].find(['{', '}']) {
            Some(close) if tail.as_bytes()[close + 1] == b'}' => close + 2,
            Some(open) => open + 1,
            None => tail.len(),
        };
        return Some(tail[..end].to_owned());
    }
    None
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    #[serde(rename = "title", default)]
    pub title: String,
    /// Draw one frame around the whole picker instead of one per pane —
    /// fzf's `--border`. Panes inside then usually set `frame: false` too,
    /// which is how a single-boxed, divided layout is written.
    #[serde(rename = "frame", default)]
    pub frame: bool,
    /// Capture the mouse: hover, click and wheel (docs/specs/21-mouse.md). Off
    /// by default, because capture takes text selection away from the
    /// terminal, and a picker that steals the mouse in a pipeline is a bug.
    #[serde(rename = "mouse", default)]
    pub mouse: bool,
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
    /// Borders that do not have focus, and hints.
    #[serde(rename = "dim", default = "default_dim")]
    pub dim: String,
    /// Drawn in an icon slot the provider left empty, so a missing glyph
    /// degrades the picture instead of shifting the text column.
    #[serde(rename = "icon_placeholder", default = "default_icon_placeholder")]
    pub icon_placeholder: String,
    /// The scrollbar's thumb (docs/specs/22-scrollbar.md). Its background in
    /// the `Background` style, its foreground glyph's colour in `Glyph`.
    #[serde(rename = "scrollbar", default = "default_scrollbar")]
    pub scrollbar: String,
    /// The track the thumb runs in: the cell background of the column the
    /// thumb is not on, or the `░` glyph's colour in `Glyph`. A slight tint
    /// over whatever the terminal's own background is, since the theme has no
    /// base colour to mix against.
    #[serde(rename = "scrollbar_bg", default = "default_scrollbar_bg")]
    pub scrollbar_bg: String,
    /// The background of the row under the pointer, with `window: (mouse:
    /// true)` (docs/specs/21-mouse.md). Quieter than `selected_bg` on
    /// purpose: hovering is not choosing.
    #[serde(rename = "hover_bg", default = "default_hover_bg")]
    pub hover_bg: String,
}

fn default_dim() -> String {
    "#6c7086".to_owned()
}

fn default_icon_placeholder() -> String {
    "\u{b7}".to_owned()
}

fn default_scrollbar() -> String {
    "#585b70".to_owned()
}

fn default_scrollbar_bg() -> String {
    "#45475a".to_owned()
}

fn default_hover_bg() -> String {
    "#313244".to_owned()
}

impl Theme {
    /// Read a theme from a RON file — the `--theme` flag's only job.
    pub fn load(path: &std::path::Path) -> Result<Self, SpecError> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| SpecError::plain(format!("{}: {error}", path.display())))?;
        crate::spec::parse_theme(&text)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: "#89b4fa".to_owned(),
            selected_bg: "#313244".to_owned(),
            text: "#cdd6f4".to_owned(),
            dim: default_dim(),
            icon_placeholder: default_icon_placeholder(),
            scrollbar: default_scrollbar(),
            scrollbar_bg: default_scrollbar_bg(),
            hover_bg: default_hover_bg(),
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
    pub fn dim_color(&self) -> Color {
        parse_color(&self.dim).unwrap_or(Color::Reset)
    }
    pub fn text_color(&self) -> Color {
        parse_color(&self.text).unwrap_or(Color::Reset)
    }
    pub fn scrollbar_color(&self) -> Color {
        parse_color(&self.scrollbar).unwrap_or(Color::Reset)
    }
    pub fn scrollbar_bg_color(&self) -> Color {
        parse_color(&self.scrollbar_bg).unwrap_or(Color::Reset)
    }
    pub fn hover_color(&self) -> Color {
        parse_color(&self.hover_bg).unwrap_or(Color::Reset)
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
    /// For `Detail`: the `Menu` pane whose selection it follows. `None`
    /// means the document's first menu.
    #[serde(rename = "from", default)]
    pub from: Option<String>,
    /// `false` draws the pane's content without the box around it.
    #[serde(rename = "frame", default = "default_true")]
    pub frame: bool,
    /// For `Menu`: draw a proportional scrollbar in the pane's last column,
    /// and give the text one cell fewer. Off by default, so no existing
    /// document changes shape.
    #[serde(rename = "scrollbar", default)]
    pub scrollbar: bool,
    /// For `Menu` with `scrollbar: true`: how the bar is drawn. Omitted means
    /// `Background` — a tinted cell, the style a terminal scrollbar has in
    /// `tmux`; `Glyph` draws `░`/`█` instead, which is the only style a
    /// plain-text snapshot or `capture-pane` can see. Both draw the same
    /// geometry from the same window (docs/specs/22-scrollbar.md).
    #[serde(rename = "scrollbar_style", default)]
    pub scrollbar_style: Option<ScrollbarStyle>,
    /// For `Counter`: how the two numbers are written. `{shown}` and
    /// `{total}` are the only placeholders there are.
    #[serde(rename = "format", default)]
    pub format: Option<String>,
    /// For `Counter`: which edge the number hugs.
    #[serde(rename = "align", default)]
    pub align: Option<Align>,
}

fn default_true() -> bool {
    true
}

/// How a `Menu`'s scrollbar is drawn, chosen per pane. Both styles take the
/// same column, light the same cells and read the same two theme tokens; only
/// what goes into those cells differs, so nothing about the geometry or the
/// window is duplicated (docs/specs/22-scrollbar.md, "Two styles, one bar").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub enum ScrollbarStyle {
    /// A tinted cell background: `scrollbar_bg` behind the track, `scrollbar`
    /// behind the thumb, no glyph. What tmux 3.8 does — its manual: "the
    /// foreground colour is used for the slider, the background for the rest of
    /// the scrollbar" — and what helix's popup does.
    #[serde(rename = "Background")]
    #[default]
    Background,
    /// A glyph in the last column: `░` for the track and `█` for the thumb,
    /// coloured by the same two tokens used as foregrounds. What ratatui's own
    /// `Scrollbar` widget ships, and the only style a text snapshot sees.
    #[serde(rename = "Glyph")]
    Glyph,
}

impl ScrollbarStyle {
    /// The name the document writes, for error messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Background => "Background",
            Self::Glyph => "Glyph",
        }
    }
}

impl Pane {
    /// The scrollbar style this pane draws with: its own, or the default.
    pub fn scrollbar_style(&self) -> ScrollbarStyle {
        self.scrollbar_style.unwrap_or_default()
    }
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
    /// A single row of hints, no frame.
    #[serde(rename = "Status")]
    Status,
    /// Draws nothing; holds its cells so a layout can push panes apart.
    #[serde(rename = "Spacer")]
    Spacer,
    /// The selected row's `detail` text: a preview that follows the menu.
    #[serde(rename = "Detail")]
    Detail,
    /// `shown/total`: how many rows survived the query, out of how many
    /// arrived. The picker writes both numbers after every refilter.
    #[serde(rename = "Counter")]
    Counter,
}

/// Where a [`Kind::Counter`] puts its number inside its rect.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub enum Align {
    #[serde(rename = "Left")]
    #[default]
    Left,
    #[serde(rename = "Right")]
    Right,
    #[serde(rename = "Center")]
    Center,
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
