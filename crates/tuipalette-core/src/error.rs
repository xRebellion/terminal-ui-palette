//! One error type for everything that can be wrong with a spec.
//!
//! Two shapes, because the two sources know different things: serde knows a
//! line and column, our own validation knows the field path. Both end up in the
//! same `Display`, so a caller never has to match on which one it got.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecError {
    pub detail: String,
    pub path: Option<String>,
    pub location: Option<Location>,
}

impl SpecError {
    pub fn plain(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
            path: None,
            location: None,
        }
    }

    pub fn at_path(path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
            path: Some(path.into()),
            location: None,
        }
    }

    pub fn at_location(location: Location, detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
            path: None,
            location: Some(location),
        }
    }
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.location, &self.path) {
            (Some(at), _) => write!(f, "line {}, column {}: {}", at.line, at.column, self.detail),
            (None, Some(path)) => write!(f, "{path}: {}", self.detail),
            (None, None) => write!(f, "{}", self.detail),
        }
    }
}

impl std::error::Error for SpecError {}
