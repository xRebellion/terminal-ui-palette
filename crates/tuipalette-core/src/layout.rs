//! Geometry: the size grammar, and the walk that hands every pane a `Rect`.
//!
//! Validation lives here too because the grammar is the thing being validated;
//! `spec::UiSpec::validate` only decides *when* to call it.

use std::collections::BTreeSet;

use ratatui_core::layout::{Constraint, Layout, Rect};

use crate::error::SpecError;
use crate::spec::{Axis, Node};

/// `"3"` cells, `"50%"`, `"fill"`, `"min:5"`, `"max:20"`.
pub fn parse_constraint(text: &str) -> Option<Constraint> {
    let spec = text.trim();
    if spec.eq_ignore_ascii_case("fill") {
        return Some(Constraint::Fill(1));
    }
    if let Some(percent) = spec.strip_suffix('%') {
        let value: u16 = percent.trim().parse().ok()?;
        return (value <= 100).then_some(Constraint::Percentage(value));
    }
    if let Some(min) = spec.strip_prefix("min:") {
        return Some(Constraint::Min(min.trim().parse().ok()?));
    }
    if let Some(max) = spec.strip_prefix("max:") {
        return Some(Constraint::Max(max.trim().parse().ok()?));
    }
    Some(Constraint::Length(spec.parse().ok()?))
}

const GRAMMAR: &str = "expected `N` cells, `N%`, `fill`, `min:N` or `max:N`";

/// Check the tree without producing geometry: shape, references, grammar.
pub fn validate(
    node: &Node,
    path: &str,
    declared: &BTreeSet<String>,
    drawn: &mut BTreeSet<String>,
) -> Result<(), SpecError> {
    match (&node.pane, node.children.is_empty()) {
        (Some(id), true) => {
            if !node.sizes.is_empty() || node.dir.is_some() || node.spacing != 0 {
                return Err(SpecError::at_path(
                    path,
                    "a leaf node takes only `pane`; `dir`, `sizes` and `spacing` belong to splits",
                ));
            }
            if !declared.contains(id) {
                return Err(SpecError::at_path(
                    format!("{path}.pane"),
                    format!("`{id}` is not a declared pane"),
                ));
            }
            if !drawn.insert(id.clone()) {
                return Err(SpecError::at_path(
                    format!("{path}.pane"),
                    format!("pane `{id}` is drawn more than once"),
                ));
            }
            Ok(())
        }
        (None, false) => {
            if node.dir.is_none() {
                return Err(SpecError::at_path(path, "a split needs `dir`"));
            }
            if !node.sizes.is_empty() && node.sizes.len() != node.children.len() {
                return Err(SpecError::at_path(
                    format!("{path}.sizes"),
                    format!(
                        "{} sizes for {} children — give one per child, or none at all",
                        node.sizes.len(),
                        node.children.len()
                    ),
                ));
            }
            for (i, size) in node.sizes.iter().enumerate() {
                if parse_constraint(size).is_none() {
                    return Err(SpecError::at_path(
                        format!("{path}.sizes[{i}]"),
                        format!("`{size}` is not a size — {GRAMMAR}"),
                    ));
                }
            }
            for (i, child) in node.children.iter().enumerate() {
                validate(child, &format!("{path}.children[{i}]"), declared, drawn)?;
            }
            Ok(())
        }
        (Some(_), false) => Err(SpecError::at_path(
            path,
            "a node is either a pane or a split, not both",
        )),
        (None, true) => Err(SpecError::at_path(
            path,
            "a node needs either `pane` or `children`",
        )),
    }
}

/// Hand every pane its rectangle, in document order.
pub fn resolve(node: &Node, area: Rect) -> Result<Vec<(String, Rect)>, SpecError> {
    let mut out = Vec::new();
    walk(node, area, "layout", &mut out)?;
    Ok(out)
}

fn walk(
    node: &Node,
    area: Rect,
    path: &str,
    out: &mut Vec<(String, Rect)>,
) -> Result<(), SpecError> {
    if let Some(id) = &node.pane {
        out.push((id.clone(), area));
        return Ok(());
    }

    let axis = node
        .dir
        .ok_or_else(|| SpecError::at_path(path, "a split needs `dir`"))?;
    let constraints = constraints(node, path)?;
    let layout = match axis {
        Axis::Vertical => Layout::vertical(constraints),
        Axis::Horizontal => Layout::horizontal(constraints),
    }
    .spacing(node.spacing);
    let rects = layout.split(area);

    for (i, (child, rect)) in node.children.iter().zip(rects.iter()).enumerate() {
        walk(child, *rect, &format!("{path}.children[{i}]"), out)?;
    }
    Ok(())
}

fn constraints(node: &Node, path: &str) -> Result<Vec<Constraint>, SpecError> {
    if node.sizes.is_empty() {
        return Ok(vec![Constraint::Fill(1); node.children.len()]);
    }
    node.sizes
        .iter()
        .enumerate()
        .map(|(i, size)| {
            parse_constraint(size).ok_or_else(|| {
                SpecError::at_path(
                    format!("{path}.sizes[{i}]"),
                    format!("`{size}` is not a size — {GRAMMAR}"),
                )
            })
        })
        .collect()
}
