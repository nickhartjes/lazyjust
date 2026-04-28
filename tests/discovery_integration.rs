use lazyjust::discovery::{discover, discover_explicit};
use std::path::{Path, PathBuf};

#[test]
fn discover_tree_returns_two_justfiles_with_recipes() {
    let root = PathBuf::from("tests/fixtures/tree");
    let result = discover(&root).unwrap();
    assert_eq!(result.justfiles.len(), 2);
    assert!(result.errors.is_empty());

    let root_abs = std::path::absolute(&root).unwrap();
    let root_jf = result
        .justfiles
        .iter()
        .find(|j| j.path == root_abs.join("justfile"))
        .unwrap();
    assert!(root_jf.recipes.iter().any(|r| r.name == "build"));
}

#[test]
fn discover_returns_absolute_paths_even_for_relative_root() {
    let root = PathBuf::from("tests/fixtures/tree");
    let result = discover(&root).unwrap();

    for jf in &result.justfiles {
        assert!(
            jf.path.is_absolute(),
            "expected absolute path, got: {}",
            jf.path.display()
        );
    }
}

#[test]
fn discover_explicit_returns_only_the_named_justfile() {
    let path = PathBuf::from("tests/fixtures/tree/sub/justfile");
    let result = discover_explicit(&path).unwrap();

    assert_eq!(result.justfiles.len(), 1);
    assert!(result.errors.is_empty());

    let jf = &result.justfiles[0];
    assert!(jf.path.is_absolute());
    assert_eq!(jf.path.file_name(), Some(Path::new("justfile").as_os_str()));
}
