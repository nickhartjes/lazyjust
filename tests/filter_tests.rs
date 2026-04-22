use lazyjust::app::filter::fuzzy_match;

#[test]
fn empty_query_keeps_all() {
    let names = ["build", "test", "deploy"];
    let scored = fuzzy_match(&names, "");
    assert_eq!(scored.len(), 3);
    // ordering preserved
    assert_eq!(scored[0].0, 0);
}

#[test]
fn query_orders_by_score() {
    let names = ["build", "test", "deploy"];
    let scored = fuzzy_match(&names, "de");
    assert!(!scored.is_empty());
    assert_eq!(names[scored[0].0], "deploy");
}

#[test]
fn query_excludes_non_matches() {
    let names = ["build", "test", "deploy"];
    let scored = fuzzy_match(&names, "zzz");
    assert!(scored.is_empty());
}
