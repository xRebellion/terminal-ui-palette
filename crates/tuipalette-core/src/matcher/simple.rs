//! The dependency-free scorer: a case-insensitive substring, ranked by how
//! early it starts. This is dmenu's behaviour, and it is what
//! `--no-default-features` falls back to when a build without nucleo matters.
//!
//! Smart case matches nucleo's: a query with an upper-case letter is matched
//! exactly, a lower-case one matches anything. The difference between the two
//! scorers is fuzziness, not case.

use super::{Match, Matcher};

pub struct SimpleMatcher {
    query: String,
    exact: bool,
}

impl SimpleMatcher {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            exact: false,
        }
    }
}

impl Default for SimpleMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for SimpleMatcher {
    fn set_query(&mut self, query: &str) {
        self.exact = query.chars().any(char::is_uppercase);
        self.query = query.to_owned();
    }

    fn score(&mut self, display: &str) -> Option<Match> {
        if self.query.is_empty() {
            return Some(Match::default());
        }
        let (needle, haystack) = if self.exact {
            (self.query.clone(), display.to_owned())
        } else {
            (self.query.to_lowercase(), display.to_lowercase())
        };
        let byte = haystack.find(&needle)?;
        let start = haystack[..byte].chars().count();
        let positions = (start..start + needle.chars().count()).collect();
        Some(Match {
            // Earlier is better. Comparable only within this scorer.
            score: 1_000u32.saturating_sub(start as u32 * 10),
            positions,
        })
    }
}
