//! How a row that matched is ranked against the others (docs/PLAN.md, 18).
//!
//! A scorer answers "does it match, and how well"; a launcher also has to
//! answer "what did the user most likely mean". fzf's algorithm, which nucleo
//! reproduces, gives a word-start bonus to every query character, so `kn`
//! scores `K`onsole's scattered `K`…`n` above the literal `kn` inside
//! Ar`kn`ights. Typing two adjacent letters is a strong statement that they are
//! adjacent, so a literal hit is put in a tier of its own above every scattered
//! one (D20):
//!
//! 1. the display **is** the query;
//! 2. the display **starts with** it;
//! 3. a **word** in the display starts with it;
//! 4. the display **contains** it;
//! 5. the display holds it as a **subsequence** (the scorer's fuzzy hit);
//! 6. only the value or the search field matches (D17).
//!
//! Inside a tier the provider's order decides. That is what makes usage a
//! ranking signal without the library knowing usage exists: `scripts/usage.sh`
//! hands the list over most-recent first, so among equally good literal hits
//! the one launched last leads. Only the fuzzy tier consults the scorer, since
//! "how scattered" is the one question a literal match does not raise.

/// What kind of match a row is, best first. `Ord` follows that order, so a
/// sort by `Tier` ascending puts the best rows on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Exact,
    Prefix,
    WordStart,
    Substring,
    Fuzzy,
    Hidden,
}

/// The literal tier of `display` for `query`, with the character indices the
/// match covers, or `None` when the query is not a substring of it.
///
/// Case follows the scorers' smart case: a query with an upper-case letter is
/// matched exactly, an all-lower-case one matches any case. An empty query is
/// not a literal hit — with nothing typed every row is equal.
pub fn literal(query: &str, display: &str) -> Option<(Tier, Vec<usize>)> {
    if query.is_empty() {
        return None;
    }
    let exact_case = query.chars().any(char::is_uppercase);
    let fold = |c: char| {
        if exact_case {
            c
        } else {
            // One char in, one char out, so indices stay character indices.
            c.to_lowercase().next().unwrap_or(c)
        }
    };
    let needle: Vec<char> = query.chars().map(fold).collect();
    let hay: Vec<char> = display.chars().map(fold).collect();
    if needle.len() > hay.len() {
        return None;
    }
    let mut best: Option<(Tier, usize)> = None;
    for start in 0..=hay.len() - needle.len() {
        if hay[start..start + needle.len()] != needle[..] {
            continue;
        }
        let tier = if start == 0 && needle.len() == hay.len() {
            Tier::Exact
        } else if start == 0 {
            Tier::Prefix
        } else if word_starts_at(&hay, start) {
            Tier::WordStart
        } else {
            Tier::Substring
        };
        if best.is_none_or(|(held, _)| tier < held) {
            best = Some((tier, start));
        }
        if tier == Tier::WordStart {
            break; // later occurrences can only tie or do worse
        }
    }
    best.map(|(tier, start)| (tier, (start..start + needle.len()).collect()))
}

/// Whether a word begins at `at`: after a non-alphanumeric character, or at a
/// lower-to-upper case change (`LibreWolf`'s `W`). The case change is read on
/// the folded text only when it still has case, which is the exact-case query.
fn word_starts_at(hay: &[char], at: usize) -> bool {
    let before = hay[at - 1];
    let here = hay[at];
    !before.is_alphanumeric() || (before.is_lowercase() && here.is_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(query: &str, display: &str) -> Option<Tier> {
        literal(query, display).map(|(tier, _)| tier)
    }

    #[test]
    fn each_literal_tier_is_told_apart() {
        assert_eq!(tier("konsole", "Konsole"), Some(Tier::Exact));
        assert_eq!(tier("kon", "Konsole"), Some(Tier::Prefix));
        assert_eq!(tier("con", "KDE Connect"), Some(Tier::WordStart));
        assert_eq!(tier("kn", "Arknights"), Some(Tier::Substring));
        assert_eq!(tier("kn", "Konsole"), None, "scattered is not literal");
        assert_eq!(tier("", "Konsole"), None, "nothing typed is no hit");
    }

    #[test]
    fn the_best_occurrence_wins_and_is_the_one_highlighted() {
        // "ark" first appears inside a word, then at one's start.
        let (tier, positions) = literal("ark", "Bookmark Ark").expect("hit");
        assert_eq!(tier, Tier::WordStart);
        assert_eq!(positions, vec![9, 10, 11]);
    }

    #[test]
    fn literal_case_is_smart_like_the_scorers() {
        assert_eq!(tier("kn", "ARKNIGHTS"), Some(Tier::Substring));
        assert_eq!(tier("KN", "Arknights"), None, "upper case is exact");
        assert_eq!(tier("Wolf", "LibreWolf"), Some(Tier::WordStart), "a case change starts a word");
    }

    #[test]
    fn a_query_longer_than_the_display_is_no_hit() {
        assert_eq!(tier("konsoles", "Konsole"), None);
    }
}
