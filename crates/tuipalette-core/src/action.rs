//! What the spec is allowed to ask for.
//!
//! The wire names are pinned on every variant: a spec file says `Quit`, and
//! renaming the Rust variant must not silently change that.
//!
//! `Input` and `Invoke` are runtime-only — a spec cannot ask for them, so they
//! are skipped by serde and never appear in a document.

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Action {
    #[serde(rename = "Quit")]
    Quit,
    #[serde(rename = "FocusNext")]
    FocusNext,
    #[serde(rename = "FocusPrev")]
    FocusPrev,
    #[serde(rename = "Up")]
    Up,
    #[serde(rename = "Down")]
    Down,
    #[serde(rename = "Accept")]
    Accept,
    #[serde(rename = "Backspace")]
    Backspace,
    /// Delete the character ahead of the cursor. The query has no cursor
    /// movement yet, so today this drops the last character, like backspace.
    #[serde(rename = "Delete")]
    Delete,
    /// A typed character. Runtime only.
    #[serde(skip)]
    Input(char),
    /// A component's command for the app. Runtime only.
    #[serde(skip)]
    Invoke(String),
}
