use lazyrust::app::types::{Param, ParamKind, Recipe};

#[test]
fn recipe_builder_basic() {
    let r = Recipe {
        name: "build".into(),
        module_path: vec![],
        group: None,
        params: vec![Param {
            name: "profile".into(),
            default: Some("debug".into()),
            kind: ParamKind::Positional,
        }],
        doc: Some("Build the project".into()),
        command_preview: "cargo build --release".into(),
        runs: vec![],
        dependencies: Vec::new(),
    };
    assert_eq!(r.params[0].kind, ParamKind::Positional);
}
