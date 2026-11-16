//! The declarative half: a RON document describes what is on screen and
//! `resolve` turns it into `Rect`s. Wire names are pinned with
//! `#[serde(rename)]`-free explicit field names and `deny_unknown_fields`, so a
//! typo fails loudly instead of silently rendering nothing.

use std::collections::BTreeMap;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use serde::Deserialize;

use crate::action::Action;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UiSpec {
    pub window: Window,
    #[serde(default)]
    pub theme: Theme,
    pub layout: Node,
    #[serde(default)]
    pub panes: Vec<Pane>,
    #[serde(default)]
    pub keys: BTreeMap<String, Option<Action>>,
}

impl UiSpec {
    pub fn parse(text: &str) -> Result<Self, String> {
        ron::from_str::<UiSpec>(text).map_err(|e| {
            format!(
                "spec error at line {}, column {}: {}",
                e.span.start.line, e.span.start.col, e.code
            )
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Theme {
    pub accent: String,
    pub selected_bg: String,
    pub text: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: "#89b4fa".into(),
            selected_bg: "#313244".into(),
            text: "#cdd6f4".into(),
        }
    }
}

impl Theme {
    pub fn accent_color(&self) -> Color {
        parse_color(&self.accent)
    }
    pub fn selected_color(&self) -> Color {
        parse_color(&self.selected_bg)
    }
    pub fn text_color(&self) -> Color {
        parse_color(&self.text)
    }
}

pub fn parse_color(spec: &str) -> Color {
    let hex = spec.trim().trim_start_matches('#');
    match hex.len() {
        6 => {
            let p = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0xff);
            Color::Rgb(p(0), p(2), p(4))
        }
        _ => Color::Reset,
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pane {
    pub id: String,
    #[serde(default)]
    pub kind: Kind,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub keys: BTreeMap<String, Option<Action>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Text,
    Input,
    Menu,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Node {
    Vertical {
        #[serde(default)]
        sizes: Vec<String>,
        #[serde(default)]
        spacing: u16,
        children: Vec<Node>,
    },
    Horizontal {
        #[serde(default)]
        sizes: Vec<String>,
        #[serde(default)]
        spacing: u16,
        children: Vec<Node>,
    },
    Pane(String),
}

/// `"3"` cells, `"50%"`, `"fill"`, `"min:5"`, `"max:20"`.
pub fn constraint(spec: &str) -> Constraint {
    let s = spec.trim();
    if s == "fill" {
        Constraint::Fill(1)
    } else if let Some(p) = s.strip_suffix('%') {
        Constraint::Percentage(p.trim().parse().unwrap_or(0))
    } else if let Some(m) = s.strip_prefix("min:") {
        Constraint::Min(m.trim().parse().unwrap_or(0))
    } else if let Some(m) = s.strip_prefix("max:") {
        Constraint::Max(m.trim().parse().unwrap_or(0))
    } else {
        Constraint::Length(s.parse().unwrap_or(0))
    }
}

pub fn resolve(node: &Node, area: Rect, out: &mut Vec<(String, Rect)>) {
    match node {
        Node::Pane(id) => out.push((id.clone(), area)),
        Node::Vertical {
            sizes,
            spacing,
            children,
        } => split(Direction::Vertical, sizes, *spacing, children, area, out),
        Node::Horizontal {
            sizes,
            spacing,
            children,
        } => split(Direction::Horizontal, sizes, *spacing, children, area, out),
    }
}

fn split(
    dir: Direction,
    sizes: &[String],
    spacing: u16,
    children: &[Node],
    area: Rect,
    out: &mut Vec<(String, Rect)>,
) {
    let constraints: Vec<Constraint> = if sizes.is_empty() {
        vec![Constraint::Fill(1); children.len()]
    } else {
        sizes.iter().map(|s| constraint(s)).collect()
    };
    let layout = if dir == Direction::Vertical {
        Layout::vertical(constraints)
    } else {
        Layout::horizontal(constraints)
    }
    .spacing(spacing);
    let rects = layout.split(area);
    for (child, rect) in children.iter().zip(rects.iter()) {
        resolve(child, *rect, out);
    }
}
