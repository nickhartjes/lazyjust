use lazyrust::discovery::walk::walk_justfiles;
use std::path::PathBuf;

#[test]
fn walk_respects_gitignore_and_defaults() {
    let root = PathBuf::from("tests/fixtures/tree");
    let found = walk_justfiles(&root).unwrap();
    let rel: Vec<_> = found
        .iter()
        .map(|p| {
            p.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert!(rel.contains(&"justfile".to_string()));
    assert!(rel.contains(&"sub/justfile".to_string()));
    assert!(!rel.iter().any(|p| p.contains("ignored_by_gitignore")));
    assert!(!rel.iter().any(|p| p.contains("node_modules")));
}
