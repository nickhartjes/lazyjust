use lazyjust::discovery::walk::walk_justfiles;
use std::path::PathBuf;

#[test]
fn walk_respects_gitignore_and_defaults() {
    let root = PathBuf::from("tests/fixtures/tree");
    let root_abs = std::path::absolute(&root).unwrap();
    let found = walk_justfiles(&root).unwrap();

    for p in &found {
        assert!(
            p.is_absolute(),
            "expected absolute path, got: {}",
            p.display()
        );
    }

    let rel: Vec<_> = found
        .iter()
        .map(|p| {
            p.strip_prefix(&root_abs)
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
