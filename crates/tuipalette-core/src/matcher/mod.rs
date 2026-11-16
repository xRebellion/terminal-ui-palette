//! Ranking items against what the user typed, and where the match sits.
//!
//! The trait is deliberately two calls — set the query once, score every item —
//! because a scorer rebuilds its pattern per keystroke, not per row. Both
//! implementations answer the same way, so the picker never knows which one it
//! has (docs/PLAN.md, D13).

pub mod simple;

#[cfg(feature = "nucleo")]
pub mod nucleo;

pub use simple::SimpleMatcher;

#[cfg(feature = "nucleo")]
pub use self::nucleo::Nucleo;

/// A row that matched: how well, and which characters to colour.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Match {
    /// Higher is better. Only compared within one scorer.
    pub score: u32,
    /// Character indices into the display string, ascending. Empty when the
    /// query is empty.
    pub positions: Vec<usize>,
}

/// The scorer. `set_query` then `score` for each visible string.
pub trait Matcher: Send {
    fn set_query(&mut self, query: &str);
    fn score(&mut self, display: &str) -> Option<Match>;
}

/// The scorer the build was compiled with: nucleo unless the feature is off.
pub fn default_matcher() -> Box<dyn Matcher> {
    #[cfg(feature = "nucleo")]
    {
        Box::new(Nucleo::new())
    }
    #[cfg(not(feature = "nucleo"))]
    {
        Box::new(SimpleMatcher::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both scorers, so every assertion is a statement about the trait rather
    /// than about one implementation.
    fn scorers() -> Vec<(&'static str, Box<dyn Matcher>)> {
        let mut out: Vec<(&'static str, Box<dyn Matcher>)> = vec![("simple", Box::new(SimpleMatcher::new()))];
        #[cfg(feature = "nucleo")]
        out.push(("nucleo", Box::new(Nucleo::new())));
        out
    }

    fn score(matcher: &mut dyn Matcher, query: &str, display: &str) -> Option<Match> {
        matcher.set_query(query);
        matcher.score(display)
    }

    #[test]
    fn the_matcher_trait_answers_the_same_way_for_both_scorers() {
        for (name, mut matcher) in scorers() {
            assert!(score(&mut *matcher, "fi", "firefox").is_some(), "{name}");
            assert!(score(&mut *matcher, "zzz", "firefox").is_none(), "{name}");
            assert!(
                score(&mut *matcher, "FIRE", "firefox").is_none(),
                "{name}: an upper-case query is matched exactly"
            );
            assert!(
                score(&mut *matcher, "fire", "FireFox").is_some(),
                "{name}: a lower-case query matches any case"
            );
        }
    }

    /// The one thing the scorers disagree about, stated on purpose: nucleo
    /// fuzzes, the fallback does not.
    #[test]
    fn only_the_nucleo_matcher_fuzzes() {
        let mut simple = SimpleMatcher::new();
        assert!(score(&mut simple, "ff", "firefox").is_none(), "substring only");
        #[cfg(feature = "nucleo")]
        {
            let mut nucleo = Nucleo::new();
            assert!(score(&mut nucleo, "ff", "firefox").is_some(), "fuzzy");
        }
    }

    #[test]
    fn a_matcher_reports_a_real_subsequence_of_the_display() {
        for (name, mut matcher) in scorers() {
            let hit = score(&mut *matcher, "fox", "firefox").unwrap_or_else(|| panic!("{name}"));
            let chars: Vec<char> = "firefox".chars().collect();
            let picked: String = hit
                .positions
                .iter()
                .map(|&index| chars[index].to_ascii_lowercase())
                .collect();
            assert_eq!(picked, "fox", "{name}");
            assert!(
                hit.positions.windows(2).all(|pair| pair[0] < pair[1]),
                "{name}: positions ascend"
            );
            assert!(hit.score > 0, "{name}");
        }
    }

    #[test]
    fn the_simple_matcher_finds_the_contiguous_run() {
        let mut matcher = SimpleMatcher::new();
        let hit = score(&mut matcher, "fox", "firefox").expect("substring");
        assert_eq!(hit.positions, vec![4, 5, 6]);
    }

    #[test]
    fn nucleo_ranks_a_prefix_higher_than_a_scattered_match() {
        #[cfg(feature = "nucleo")]
        {
            let mut matcher = Nucleo::new();
            let prefix = score(&mut matcher, "fire", "firefox").expect("prefix");
            let scattered = score(&mut matcher, "fire", "f-i-r-e").expect("scattered");
            assert!(prefix.score > scattered.score, "{} vs {}", prefix.score, scattered.score);
        }
    }

    #[test]
    fn an_empty_query_matches_everything_with_no_spans() {
        for (name, mut matcher) in scorers() {
            let hit = score(&mut *matcher, "", "anything").unwrap_or_else(|| panic!("{name}"));
            assert!(hit.positions.is_empty(), "{name}");
        }
    }

    #[test]
    fn the_default_matcher_is_the_compiled_one() {
        let mut matcher = default_matcher();
        assert!(score(&mut *matcher, "ff", "firefox").is_some());
    }
}
