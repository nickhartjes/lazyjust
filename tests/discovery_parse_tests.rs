use lazyjust::app::types::{ParamKind, Recipe};
use lazyjust::discovery::parse::parse_dump;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/dumps/{name}.json"))
        .expect("fixture exists")
}

#[test]
fn parse_simple() {
    let recipes = parse_dump(&fixture("simple")).unwrap();
    let names: Vec<_> = recipes.iter().map(|r: &Recipe| r.name.clone()).collect();
    assert!(names.contains(&"build".to_string()));
    assert!(names.contains(&"test".to_string()));
    let build = recipes.iter().find(|r| r.name == "build").unwrap();
    assert_eq!(build.params.len(), 0);
    assert_eq!(build.group, None);
}

#[test]
fn parse_with_params() {
    let recipes = parse_dump(&fixture("with_params")).unwrap();
    let deploy = recipes.iter().find(|r| r.name == "deploy").unwrap();
    assert_eq!(deploy.params.len(), 1);
    assert_eq!(deploy.params[0].name, "env");
    assert_eq!(deploy.params[0].default.as_deref(), Some("staging"));
    assert_eq!(deploy.params[0].kind, ParamKind::Positional);
    assert_eq!(deploy.doc.as_deref(), Some("Deploy to an environment"));

    let notify = recipes.iter().find(|r| r.name == "notify").unwrap();
    assert_eq!(notify.params[0].kind, ParamKind::Variadic);
}

#[test]
fn parse_with_groups() {
    let recipes = parse_dump(&fixture("with_groups")).unwrap();
    let test = recipes.iter().find(|r| r.name == "test").unwrap();
    assert_eq!(test.group.as_deref(), Some("ci"));
    let deploy = recipes.iter().find(|r| r.name == "deploy").unwrap();
    assert_eq!(deploy.group.as_deref(), Some("deploy"));
}
