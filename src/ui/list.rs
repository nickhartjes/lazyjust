use crate::app::types::{ListMode, Recipe};
use crate::app::view::RowRef;
use crate::app::App;
use crate::ui::focus::is_list_active;
use crate::ui::icon_style::IconStyle;
use crate::ui::path_relativize::relativize_to_root;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let _ = is_list_active(app.focus);
    let lines = build_lines(app, theme, area.width);
    f.render_widget(Paragraph::new(lines), area);
}

#[cfg(test)]
pub(crate) fn build_lines_for_test<'a>(app: &'a App, width: u16) -> Vec<Line<'a>> {
    build_lines(app, &app.theme, width)
}

fn build_lines<'a>(
    app: &'a App,
    theme: &crate::theme::Theme,
    width: u16,
) -> Vec<Line<'a>> {
    let glyphs = app.icon_style.glyphs();
    let style = app.icon_style;
    let g = &glyphs;

    // Flatten the view into a Vec of (jf_idx, recipe_idx) keyed by recipe
    // position. This is the same indexing space the filter uses.
    let positions: Vec<(usize, usize)> = app
        .view
        .rows
        .iter()
        .filter_map(|r| match r {
            RowRef::Recipe { jf_idx, recipe_idx } => Some((*jf_idx, *recipe_idx)),
            RowRef::Header { .. } => None,
        })
        .collect();

    let names: Vec<&str> = positions
        .iter()
        .map(|(jf, ri)| app.justfiles[*jf].recipes[*ri].name.as_str())
        .collect();

    let scored = crate::app::filter::fuzzy_match(&names, app.filter.as_str());
    let selected = app.list_cursor.min(scored.len().saturating_sub(1));

    if app.list_mode == ListMode::All {
        build_all_mode(&positions, &scored, app, theme, style, g, width, selected)
    } else {
        build_active_mode(&positions, &scored, app, theme, style, g, width, selected)
    }
}

fn build_active_mode<'a>(
    positions: &[(usize, usize)],
    scored: &[(usize, u32)],
    app: &'a App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
    width: u16,
    selected: usize,
) -> Vec<Line<'a>> {
    let mut out: Vec<Line> = Vec::new();
    let mut current_group: Option<&str> = None;
    for (displayed_idx, (pos, _score)) in scored.iter().enumerate() {
        let (jf_idx, recipe_idx) = positions[*pos];
        let r: &Recipe = &app.justfiles[jf_idx].recipes[recipe_idx];
        let group_name = r.group.as_deref();
        if group_name != current_group {
            let lbl = group_name.unwrap_or("RECIPES").to_ascii_uppercase();
            out.push(section_header(&lbl, theme, width));
            current_group = group_name;
        }
        let is_cursor = displayed_idx == selected;
        out.push(row(r, app, theme, style, g, is_cursor, width));
    }
    out
}

fn build_all_mode<'a>(
    positions: &[(usize, usize)],
    scored: &[(usize, u32)],
    app: &'a App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
    width: u16,
    selected: usize,
) -> Vec<Line<'a>> {
    use std::collections::HashMap;
    // Group surviving recipe positions by jf_idx, preserving the score
    // ordering inside each group.
    let mut by_jf: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut jf_order: Vec<usize> = Vec::new();
    for (pos, _score) in scored {
        let (jf_idx, _recipe_idx) = positions[*pos];
        by_jf
            .entry(jf_idx)
            .and_modify(|v| v.push(*pos))
            .or_insert_with(|| {
                jf_order.push(jf_idx);
                vec![*pos]
            });
    }

    // Iterate justfiles in view order so the section ordering follows
    // discovery's path-sorted layout. Drop justfiles with no surviving
    // recipes.
    let mut out: Vec<Line> = Vec::new();
    let mut emitted = 0usize;
    let view_jf_order: Vec<usize> = app
        .view
        .rows
        .iter()
        .filter_map(|r| match r {
            RowRef::Header { jf_idx } => Some(*jf_idx),
            _ => None,
        })
        .collect();
    for jf_idx in view_jf_order {
        let Some(survivors) = by_jf.get(&jf_idx) else {
            continue;
        };
        let label = relativize_to_root(
            &app.justfiles[jf_idx].path,
            &app.discovery_root,
        );
        out.push(section_header(&label, theme, width));
        for pos in survivors {
            let (jf, recipe_idx) = positions[*pos];
            debug_assert_eq!(jf, jf_idx);
            let r: &Recipe = &app.justfiles[jf].recipes[recipe_idx];
            let is_cursor = emitted == selected;
            emitted += 1;
            out.push(row(r, app, theme, style, g, is_cursor, width));
        }
    }
    out
}

fn section_header<'a>(label: &str, theme: &crate::theme::Theme, width: u16) -> Line<'a> {
    let title = format!(" {label} ");
    let rule_len = width.saturating_sub(title.chars().count() as u16);
    let rule: String = "─".repeat(rule_len as usize);
    Line::from(vec![
        Span::styled(title, Style::default().fg(theme.accent)),
        Span::styled(rule, Style::default().fg(theme.dim)),
    ])
}

fn row<'a>(
    r: &'a Recipe,
    app: &App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
    is_cursor: bool,
    width: u16,
) -> Line<'a> {
    let glyph = if style == IconStyle::None {
        if is_cursor {
            "▶"
        } else {
            " "
        }
    } else if is_cursor {
        g.cursor
    } else {
        g.unselected
    };
    let leading = format!(" {glyph} ");
    let name_style = if is_cursor {
        Style::default()
            .fg(theme.selected_fg)
            .bg(theme.highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };
    let row_bg = if is_cursor {
        Some(theme.highlight)
    } else {
        None
    };

    let mut spans = vec![Span::styled(leading.clone(), name_style)];
    spans.push(Span::styled(r.name.clone(), name_style));
    spans.extend(session_indicators_for(r, app, theme, style, g));

    if r.has_deps() {
        let deps = dep_line(
            r,
            width
                .saturating_sub(visible_width(&spans) as u16)
                .saturating_sub(5),
        );
        if !deps.is_empty() {
            spans.push(Span::styled(
                format!("   → {deps}"),
                Style::default().fg(theme.dim),
            ));
        }
    }

    if let Some(bg) = row_bg {
        for s in spans.iter_mut() {
            s.style = s.style.bg(bg);
        }
        let used = visible_width(&spans) as u16;
        if used < width {
            spans.push(Span::styled(
                " ".repeat((width - used) as usize),
                Style::default().bg(bg),
            ));
        }
    }
    Line::from(spans)
}

fn dep_line(r: &Recipe, avail: u16) -> String {
    let joined = r.dep_names().join(" · ");
    if (joined.chars().count() as u16) <= avail {
        joined
    } else {
        let mut acc = String::new();
        for ch in joined.chars() {
            if acc.chars().count() as u16 + 1 >= avail.saturating_sub(1) {
                break;
            }
            acc.push(ch);
        }
        acc.push('…');
        acc
    }
}

fn visible_width(spans: &[Span]) -> usize {
    spans.iter().map(|s| s.content.chars().count()).sum()
}

fn session_indicators_for<'a>(
    r: &'a Recipe,
    app: &App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
) -> Vec<Span<'a>> {
    let mut out = Vec::new();
    let mut emitted = 0usize;
    for &sid in r.runs.iter().rev() {
        if emitted >= 3 {
            out.push(Span::styled(
                format!("  +{}", r.runs.len() - 3),
                Style::default().fg(theme.dim),
            ));
            break;
        }
        if let Some(s) = app.session(sid) {
            out.push(Span::raw("  "));
            out.push(status_span(s.status, s.unread, theme, style, g));
            emitted += 1;
        }
    }
    out
}

fn status_span(
    status: crate::app::types::Status,
    unread: bool,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
) -> Span<'static> {
    let (icon, color) = match status {
        crate::app::types::Status::Running => (
            if style == IconStyle::None {
                ""
            } else {
                g.running
            },
            theme.running,
        ),
        crate::app::types::Status::ShellAfterExit { code } | crate::app::types::Status::Exited { code } if code == 0 => {
            ("✓", if unread { theme.success } else { theme.dim })
        }
        crate::app::types::Status::ShellAfterExit { .. } | crate::app::types::Status::Exited { .. } => {
            ("✗", if unread { theme.error } else { theme.dim })
        }
        crate::app::types::Status::Broken => ("!", theme.warn),
    };
    Span::styled(icon.to_string(), Style::default().fg(color))
}

#[cfg(test)]
mod tests {
    use crate::app::types::{Justfile, ListMode, Recipe};
    use crate::app::App;
    use crate::ui::icon_style::IconStyle;
    use std::path::PathBuf;

    fn r(n: &str) -> Recipe {
        Recipe {
            name: n.into(),
            module_path: vec![],
            group: None,
            params: vec![],
            doc: None,
            command_preview: String::new(),
            runs: vec![],
            dependencies: vec![],
        }
    }

    fn make_app(mode: ListMode) -> App {
        let a = Justfile {
            path: PathBuf::from("/root/api/justfile"),
            recipes: vec![r("build"), r("test")],
            groups: vec![],
        };
        let b = Justfile {
            path: PathBuf::from("/root/web/justfile"),
            recipes: vec![r("dev")],
            groups: vec![],
        };
        App::new(
            vec![a, b],
            vec![],
            0.3,
            crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME),
            crate::theme::DEFAULT_THEME_NAME.to_string(),
            IconStyle::Round,
            mode,
            PathBuf::from("/root"),
        )
    }

    #[test]
    fn all_mode_emits_per_justfile_headers_with_relative_path_labels() {
        let app = make_app(ListMode::All);
        let lines = super::build_lines_for_test(&app, 80);
        // First line should be the header for justfile A
        let first = render_to_plain(&lines[0]);
        assert!(
            first.contains("API/JUSTFILE") || first.contains("api/justfile"),
            "expected api/justfile in first header, got {first:?}"
        );
        // Somewhere in the lines should be a header for justfile B
        let any_b = lines
            .iter()
            .map(render_to_plain)
            .any(|s| s.to_lowercase().contains("web/justfile"));
        assert!(any_b, "expected a header for web/justfile");
    }

    #[test]
    fn active_mode_renders_recipes_for_active_justfile_only() {
        let app = make_app(ListMode::Active);
        let lines = super::build_lines_for_test(&app, 80);
        let plain: Vec<String> = lines.iter().map(render_to_plain).collect();
        let joined = plain.join("\n");
        assert!(joined.contains("build"), "expected build in {joined}");
        assert!(joined.contains("test"), "expected test in {joined}");
        assert!(!joined.contains("dev"), "should not contain web's dev recipe");
    }

    fn render_to_plain(line: &ratatui::text::Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }
}
