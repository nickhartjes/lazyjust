use crate::app::types::{Recipe, Status};
use crate::app::App;
use crate::ui::focus::{focus_bar, is_list_active};
use crate::ui::icon_style::IconStyle;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App, theme: &crate::theme::Theme) {
    let active = is_list_active(app.focus);
    let Some(jf) = app.active_justfile() else {
        f.render_widget(Paragraph::new(Line::from(focus_bar(active, theme))), area);
        return;
    };

    let glyphs = app.icon_style.glyphs();
    let lines = build_lines(
        jf.recipes.as_slice(),
        &app.filter,
        app,
        theme,
        app.icon_style,
        &glyphs,
        active,
        area.width,
    );
    f.render_widget(Paragraph::new(lines), area);
}

#[allow(clippy::too_many_arguments)]
fn build_lines<'a>(
    recipes: &'a [Recipe],
    filter: &str,
    app: &App,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
    active: bool,
    width: u16,
) -> Vec<Line<'a>> {
    let names: Vec<&str> = recipes.iter().map(|r| r.name.as_str()).collect();
    let scored = crate::app::filter::fuzzy_match(&names, filter);

    let mut out: Vec<Line> = Vec::new();
    let mut current_group: Option<&str> = None;
    let mut section_count = 0usize;
    let selected = app.list_cursor.min(scored.len().saturating_sub(1));

    for (displayed_idx, (idx, _score)) in scored.iter().enumerate() {
        let r = &recipes[*idx];
        let group_name = r.group.as_deref();
        if group_name != current_group {
            let label = group_name.unwrap_or("RECIPES").to_ascii_uppercase();
            out.push(section_header(&label, theme, width, active, section_count == 0));
            section_count += 1;
            current_group = group_name;
        }
        let is_cursor = displayed_idx == selected;
        out.push(row(r, app, theme, style, g, is_cursor, width));
    }
    out
}

fn section_header<'a>(label: &str, theme: &crate::theme::Theme, width: u16, active: bool, first: bool) -> Line<'a> {
    let bar = crate::ui::focus::focus_bar(active && first, theme);
    let title = format!(" {label} ");
    let used = 1 + title.chars().count() as u16;
    let rule_len = width.saturating_sub(used);
    let rule: String = "─".repeat(rule_len as usize);
    Line::from(vec![
        bar,
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
    let (marker, bullet) = if is_cursor { (g.cursor, "") } else { ("", g.unselected) };
    let leading = if style == IconStyle::None {
        if is_cursor { "▶  ".to_string() } else { "   ".to_string() }
    } else if is_cursor {
        format!("{marker}  ")
    } else {
        format!("   {bullet} ")
    };
    let name_style = if is_cursor {
        Style::default().fg(theme.selected_fg).bg(theme.highlight).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };
    let row_bg = if is_cursor { Some(theme.highlight) } else { None };

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
            spans.push(Span::styled(format!("   → {deps}"), Style::default().fg(theme.dim)));
        }
    }

    if let Some(bg) = row_bg {
        for s in spans.iter_mut() {
            s.style = s.style.bg(bg);
        }
        let used = visible_width(&spans) as u16;
        if used < width {
            spans.push(Span::styled(" ".repeat((width - used) as usize), Style::default().bg(bg)));
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
            if acc.chars().count() as u16 + 1 >= avail.saturating_sub(1) { break; }
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
    status: Status,
    unread: bool,
    theme: &crate::theme::Theme,
    style: IconStyle,
    g: &crate::ui::icon_style::Glyphs,
) -> Span<'static> {
    let (icon, color) = match status {
        Status::Running => (if style == IconStyle::None { "" } else { g.running }, theme.running),
        Status::ShellAfterExit { code } | Status::Exited { code } if code == 0 => {
            ("✓", if unread { theme.success } else { theme.dim })
        }
        Status::ShellAfterExit { .. } | Status::Exited { .. } => {
            ("✗", if unread { theme.error } else { theme.dim })
        }
        Status::Broken => ("!", theme.warn),
    };
    Span::styled(icon.to_string(), Style::default().fg(color))
}
