use lazyrust::discovery::discover;
use std::path::PathBuf;

#[test]
fn discover_tree_returns_two_justfiles_with_recipes() {
    let root = PathBuf::from("tests/fixtures/tree");
    let result = discover(&root).unwrap();
    assert_eq!(result.justfiles.len(), 2);
    assert!(result.errors.is_empty());

    let root_jf = result
        .justfiles
        .iter()
        .find(|j| j.path == root.join("justfile"))
        .unwrap();
    assert!(root_jf.recipes.iter().any(|r| r.name == "build"));
}
