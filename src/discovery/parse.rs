use crate::app::types::{Param, ParamKind, Recipe};
use crate::error::{Error, Result};
use serde::Deserialize;
use std::path::PathBuf;

pub fn parse_dump(json: &str) -> Result<Vec<Recipe>> {
    parse_dump_with_path(json, &PathBuf::from("<unknown>"))
}

#[allow(clippy::ptr_arg)] // `&PathBuf` is part of the stable public API; see plan.
pub fn parse_dump_with_path(json: &str, path: &PathBuf) -> Result<Vec<Recipe>> {
    let raw: RawDump = serde_json::from_str(json).map_err(|e| Error::JustDumpParse {
        path: path.clone(),
        source: e,
    })?;
    let mut recipes: Vec<Recipe> = raw
        .recipes
        .into_iter()
        .filter(|(_, r)| !r.private)
        .map(|(name, r)| Recipe {
            name,
            module_path: Vec::new(),
            group: extract_group(&r.attributes),
            params: r.parameters.into_iter().map(convert_param).collect(),
            doc: r.doc.filter(|s| !s.is_empty()),
            command_preview: render_body(&r.body),
            runs: Vec::new(),
            dependencies: r.dependencies.into_iter().map(|d| d.recipe).collect(),
        })
        .collect();
    recipes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(recipes)
}

fn convert_param(p: RawParam) -> Param {
    let kind = match p.kind.as_str() {
        "plus" | "star" => ParamKind::Variadic,
        _ => ParamKind::Positional,
    };
    Param {
        name: p.name,
        default: p.default,
        kind,
    }
}

fn extract_group(attrs: &[serde_json::Value]) -> Option<String> {
    for a in attrs {
        if let Some(obj) = a.as_object() {
            if let Some(v) = obj.get("group").and_then(|g| g.as_str()) {
                return Some(v.to_string());
            }
        } else if let Some(s) = a.as_str() {
            // Some just versions emit attributes as strings like "group('ci')".
            if let Some(rest) = s.strip_prefix("group(") {
                let rest = rest.trim_end_matches(')');
                let rest = rest.trim_matches(|c| c == '\'' || c == '"');
                return Some(rest.to_string());
            }
        } else {
            // Other JSON value types (number, bool, null, array) are not group attributes.
        }
    }
    None
}

/// Render a recipe body to a preview string.
///
/// `just --dump --dump-format=json` can emit `body` in several shapes
/// depending on version:
///   * `Vec<String>` — each element is a full line (older just).
///   * `Vec<Vec<Value>>` — each outer element is a line; each inner element
///     is either a literal string (`"cargo build"`) or an interpolation
///     fragment such as `["variable", "env"]` (just 1.48+).
///
/// We tolerate both by flattening fragments to their string form for
/// interpolations. The preview is intentionally lossy — it is only used for
/// UI display.
fn render_body(body: &[serde_json::Value]) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(body.len());
    for line in body {
        match line {
            serde_json::Value::String(s) => lines.push(s.clone()),
            serde_json::Value::Array(fragments) => {
                let mut buf = String::new();
                for frag in fragments {
                    match frag {
                        serde_json::Value::String(s) => buf.push_str(s),
                        serde_json::Value::Array(parts) => {
                            // interpolation like ["variable", "env"] — render as {{name}}
                            let name = parts.iter().filter_map(|p| p.as_str()).nth(1).unwrap_or("");
                            buf.push_str("{{");
                            buf.push_str(name);
                            buf.push_str("}}");
                        }
                        _ => {}
                    }
                }
                lines.push(buf);
            }
            _ => {}
        }
    }
    lines.join("\n")
}

#[derive(Deserialize)]
struct RawDump {
    recipes: std::collections::BTreeMap<String, RawRecipe>,
}

#[derive(Deserialize)]
struct RawRecipe {
    #[serde(default)]
    parameters: Vec<RawParam>,
    #[serde(default)]
    body: Vec<serde_json::Value>,
    #[serde(default)]
    doc: Option<String>,
    #[serde(default)]
    attributes: Vec<serde_json::Value>,
    #[serde(default)]
    private: bool,
    #[serde(default)]
    dependencies: Vec<RawDep>,
}

#[derive(Deserialize)]
struct RawDep {
    recipe: String,
}

#[derive(Deserialize)]
struct RawParam {
    name: String,
    #[serde(default)]
    default: Option<String>,
    #[serde(default = "default_kind")]
    kind: String,
}

fn default_kind() -> String {
    "singular".into()
}
