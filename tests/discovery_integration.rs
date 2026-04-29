use lazyjust::discovery::{discover, DiscoverOptions};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from("tests/fixtures/tree")
}

#[test]
fn discover_tree_returns_two_justfiles_with_recipes() {
    let root = fixture_root();
    let result = discover(DiscoverOptions {
        path: Some(&root),
        justfile: None,
    })
    .unwrap();
    assert_eq!(result.justfiles.len(), 2);
    assert!(result.errors.is_empty());

    let root_abs = std::path::absolute(&root).unwrap();
    let root_jf = result
        .justfiles
        .iter()
        .find(|j| j.path == root_abs.join("justfile"))
        .unwrap();
    assert!(root_jf.recipes.iter().any(|r| r.name == "build"));
    assert_eq!(result.active_index, 0);
}

#[test]
fn discover_returns_absolute_paths_even_for_relative_root() {
    let root = fixture_root();
    let result = discover(DiscoverOptions {
        path: Some(&root),
        justfile: None,
    })
    .unwrap();

    for jf in &result.justfiles {
        assert!(
            jf.path.is_absolute(),
            "expected absolute path, got: {}",
            jf.path.display()
        );
    }
}

#[test]
fn justfile_only_walks_parent_and_pins_active() {
    let pin = fixture_root().join("sub").join("justfile");
    let result = discover(DiscoverOptions {
        path: None,
        justfile: Some(&pin),
    })
    .unwrap();

    let pin_abs = std::path::absolute(&pin).unwrap();

    // The walk root is `tests/fixtures/tree/sub`, which only contains the
    // sub/justfile — no siblings. So the result is exactly one entry.
    assert_eq!(result.justfiles.len(), 1, "got: {:?}", result.justfiles);
    assert_eq!(result.justfiles[0].path, pin_abs);
    assert_eq!(result.active_index, 0);
}

#[test]
fn path_plus_justfile_unions_walks_and_pins_active() {
    let path = fixture_root();
    let pin = fixture_root().join("sub").join("justfile");
    let result = discover(DiscoverOptions {
        path: Some(&path),
        justfile: Some(&pin),
    })
    .unwrap();

    // Walking `tree` finds `tree/justfile` and `tree/sub/justfile` (the
    // node_modules/* and ignored_by_gitignore/* are excluded by the
    // hardcoded skip list and `.gitignore` respectively). Walking
    // `tree/sub` finds the same `tree/sub/justfile`. After dedup we
    // expect exactly the same two justfiles.
    assert_eq!(result.justfiles.len(), 2);
    assert!(result.errors.is_empty());

    let pin_abs = std::path::absolute(&pin).unwrap();
    assert_eq!(
        result.justfiles[result.active_index].path, pin_abs,
        "active_index should point at the pinned justfile"
    );
}

#[test]
fn justfile_outside_walked_tree_is_force_included() {
    // The fixture's `ignored_by_gitignore/justfile` would be skipped by
    // a normal walk of `tests/fixtures/tree`. Pinning it must still
    // surface it in the result and pin it active.
    let path = fixture_root();
    let pin = fixture_root().join("ignored_by_gitignore").join("justfile");

    let result = discover(DiscoverOptions {
        path: Some(&path),
        justfile: Some(&pin),
    })
    .unwrap();

    let pin_abs = std::path::absolute(&pin).unwrap();
    assert!(
        result.justfiles.iter().any(|j| j.path == pin_abs),
        "expected gitignored pin to appear in result"
    );
    assert_eq!(
        result.justfiles[result.active_index].path, pin_abs,
        "active_index should point at the gitignored pin"
    );
}

#[test]
fn no_args_defaults_to_cwd() {
    let result = discover(DiscoverOptions::default()).unwrap();
    // We can't assert what's in cwd — just that the call succeeds and
    // active_index is in range.
    assert!(
        result.active_index <= result.justfiles.len(),
        "active_index out of range",
    );
}
