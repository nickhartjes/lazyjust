//! Single ordered list of rows shared by the list renderer, the cursor
//! reducer, the filter, and the run-dispatch path. See
//! `docs/superpowers/specs/2026-04-29-list-mode-design.md`.

use crate::app::types::{Justfile, ListMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowRef {
    /// Section header — only emitted in `ListMode::All`.
    Header { jf_idx: usize },
    /// One recipe row. `jf_idx` indexes `App.justfiles`; `recipe_idx`
    /// indexes that justfile's `recipes` vec.
    Recipe { jf_idx: usize, recipe_idx: usize },
}

#[derive(Debug, Default, Clone)]
pub struct ListView {
    pub rows: Vec<RowRef>,
}

impl ListView {
    pub fn build(justfiles: &[Justfile], mode: ListMode, active_jf: usize) -> Self {
        let rows = match mode {
            ListMode::Active => build_active(justfiles, active_jf),
            ListMode::All => build_all(justfiles),
        };
        Self { rows }
    }

    /// Number of `Recipe` rows in this view (headers excluded).
    pub fn recipe_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|r| matches!(r, RowRef::Recipe { .. }))
            .count()
    }

    /// Resolve cursor index `n` (counted over Recipe rows only) to the
    /// underlying `(jf_idx, recipe_idx)` pair.
    pub fn recipe_at(&self, n: usize) -> Option<(usize, usize)> {
        self.rows
            .iter()
            .filter_map(|r| match r {
                RowRef::Recipe { jf_idx, recipe_idx } => Some((*jf_idx, *recipe_idx)),
                _ => None,
            })
            .nth(n)
    }
}

fn build_active(justfiles: &[Justfile], active_jf: usize) -> Vec<RowRef> {
    let Some(jf) = justfiles.get(active_jf) else {
        return Vec::new();
    };
    (0..jf.recipes.len())
        .map(|i| RowRef::Recipe {
            jf_idx: active_jf,
            recipe_idx: i,
        })
        .collect()
}

fn build_all(justfiles: &[Justfile]) -> Vec<RowRef> {
    // `discovery::discover` already sorts by path; preserve that order.
    let mut out = Vec::new();
    for (jf_idx, jf) in justfiles.iter().enumerate() {
        if jf.recipes.is_empty() {
            continue;
        }
        out.push(RowRef::Header { jf_idx });
        for recipe_idx in 0..jf.recipes.len() {
            out.push(RowRef::Recipe { jf_idx, recipe_idx });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::types::{Justfile, Recipe};
    use std::path::PathBuf;

    fn recipe(name: &str) -> Recipe {
        Recipe {
            name: name.into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: String::new(),
            runs: vec![],
            dependencies: vec![],
        }
    }

    fn jf(path: &str, names: &[&str]) -> Justfile {
        Justfile {
            path: PathBuf::from(path),
            recipes: names.iter().map(|n| recipe(n)).collect(),
            groups: vec![],
        }
    }

    #[test]
    fn active_mode_emits_recipe_rows_for_active_justfile_only() {
        let files = vec![jf("a/justfile", &["x", "y"]), jf("b/justfile", &["z"])];
        let v = ListView::build(&files, ListMode::Active, 0);
        assert_eq!(
            v.rows,
            vec![
                RowRef::Recipe {
                    jf_idx: 0,
                    recipe_idx: 0
                },
                RowRef::Recipe {
                    jf_idx: 0,
                    recipe_idx: 1
                },
            ]
        );
    }

    #[test]
    fn active_mode_with_invalid_active_index_is_empty() {
        let files = vec![jf("a/justfile", &["x"])];
        let v = ListView::build(&files, ListMode::Active, 99);
        assert!(v.rows.is_empty());
    }

    #[test]
    fn all_mode_emits_header_then_recipes_per_justfile() {
        let files = vec![jf("a/justfile", &["x", "y"]), jf("b/justfile", &["z"])];
        let v = ListView::build(&files, ListMode::All, 0);
        assert_eq!(
            v.rows,
            vec![
                RowRef::Header { jf_idx: 0 },
                RowRef::Recipe {
                    jf_idx: 0,
                    recipe_idx: 0
                },
                RowRef::Recipe {
                    jf_idx: 0,
                    recipe_idx: 1
                },
                RowRef::Header { jf_idx: 1 },
                RowRef::Recipe {
                    jf_idx: 1,
                    recipe_idx: 0
                },
            ]
        );
    }

    #[test]
    fn all_mode_skips_zero_recipe_justfiles() {
        let files = vec![
            jf("a/justfile", &["x"]),
            jf("b/justfile", &[]),
            jf("c/justfile", &["y"]),
        ];
        let v = ListView::build(&files, ListMode::All, 0);
        assert_eq!(
            v.rows,
            vec![
                RowRef::Header { jf_idx: 0 },
                RowRef::Recipe {
                    jf_idx: 0,
                    recipe_idx: 0
                },
                RowRef::Header { jf_idx: 2 },
                RowRef::Recipe {
                    jf_idx: 2,
                    recipe_idx: 0
                },
            ]
        );
    }

    #[test]
    fn recipe_at_skips_headers() {
        let files = vec![jf("a/justfile", &["x", "y"]), jf("b/justfile", &["z"])];
        let v = ListView::build(&files, ListMode::All, 0);
        assert_eq!(v.recipe_at(0), Some((0, 0)));
        assert_eq!(v.recipe_at(1), Some((0, 1)));
        assert_eq!(v.recipe_at(2), Some((1, 0)));
        assert_eq!(v.recipe_at(3), None);
    }

    #[test]
    fn recipe_count_excludes_headers() {
        let files = vec![jf("a/justfile", &["x"]), jf("b/justfile", &["y"])];
        let v_active = ListView::build(&files, ListMode::Active, 0);
        assert_eq!(v_active.recipe_count(), 1);
        let v_all = ListView::build(&files, ListMode::All, 0);
        assert_eq!(v_all.recipe_count(), 2);
    }

    #[test]
    fn active_mode_uses_jf_idx_of_active_justfile_not_zero() {
        let files = vec![jf("a/justfile", &["x"]), jf("b/justfile", &["y", "z"])];
        let v = ListView::build(&files, ListMode::Active, 1);
        assert_eq!(
            v.rows,
            vec![
                RowRef::Recipe {
                    jf_idx: 1,
                    recipe_idx: 0
                },
                RowRef::Recipe {
                    jf_idx: 1,
                    recipe_idx: 1
                },
            ]
        );
    }
}
