//! The default scorer, wrapping `nucleo-matcher` for fzf-compatible ranking.
//!
//! It is adapted, not re-exported: the rest of the library only ever sees
//! `Matcher` and `Match` (docs/PLAN.md, D13).

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher as NucleoMatcher, Utf32Str};

use super::{Match, Matcher};

pub struct Nucleo {
    matcher: NucleoMatcher,
    pattern: Option<Pattern>,
    buffer: Vec<char>,
    indices: Vec<u32>,
}

impl Nucleo {
    pub fn new() -> Self {
        Self {
            matcher: NucleoMatcher::new(Config::DEFAULT),
            pattern: None,
            buffer: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl Default for Nucleo {
    fn default() -> Self {
        Self::new()
    }
}

impl Matcher for Nucleo {
    fn set_query(&mut self, query: &str) {
        self.pattern = if query.trim().is_empty() {
            None
        } else {
            Some(Pattern::parse(
                query,
                CaseMatching::Smart,
                Normalization::Smart,
            ))
        };
    }

    fn score(&mut self, display: &str) -> Option<Match> {
        let Some(pattern) = &self.pattern else {
            return Some(Match::default());
        };
        self.indices.clear();
        let haystack = Utf32Str::new(display, &mut self.buffer);
        let score = pattern.indices(haystack, &mut self.matcher, &mut self.indices)?;
        // nucleo returns the indices in match order, not sorted.
        self.indices.sort_unstable();
        self.indices.dedup();
        Some(Match {
            score,
            positions: self.indices.iter().map(|index| *index as usize).collect(),
        })
    }
}
