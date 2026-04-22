use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Config, Matcher, Utf32Str};

/// Fuzzy-match `items` against `query`.
///
/// Returns `(index, score)` pairs.
/// * Empty query: all items, score 0, in original index order.
/// * Non-empty query: only matches, sorted by score descending,
///   ties broken by original index ascending.
pub fn fuzzy_match(items: &[&str], query: &str) -> Vec<(usize, u32)> {
    if query.is_empty() {
        return items.iter().enumerate().map(|(i, _)| (i, 0)).collect();
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut buf: Vec<char> = Vec::new();
    let mut scored: Vec<(usize, u32)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            buf.clear();
            let haystack = Utf32Str::new(s, &mut buf);
            pattern
                .score(haystack, &mut matcher)
                .map(|score| (i, score))
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored
}
