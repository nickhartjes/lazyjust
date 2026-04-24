use crate::app::types::{SessionMeta, Status};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(
    f: &mut Frame,
    area: Rect,
    meta: &SessionMeta,
    _active: bool,
    theme: &crate::theme::Theme,
) {
    let elapsed = fmt_elapsed(meta.started_at.elapsed());
    let (glyph, glyph_color, label) = match meta.status {
        Status::Running => ("●", theme.running, format!("running · {elapsed}")),
        Status::Exited { code: 0 } => ("✓", theme.success, format!("done · {elapsed}")),
        Status::Exited { code } => ("✗", theme.error, format!("exit {code} · {elapsed}")),
        Status::ShellAfterExit { code } => (
            "⌁",
            theme.info,
            format!("shell (exited {code}) · press ^D to close"),
        ),
        Status::Broken => ("!", theme.warn, "broken".into()),
    };

    let mut left: Vec<Span> = vec![
        Span::styled(
            meta.recipe_name.clone(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(glyph, Style::default().fg(glyph_color)),
        Span::raw(" "),
        Span::styled(label, Style::default().fg(theme.dim)),
    ];

    let pid_text = meta
        .pid
        .map(|p| format!("pid {p} · logs ↗"))
        .unwrap_or_else(|| "logs ↗".into());
    let right_w = pid_text.chars().count();
    let left_w: usize = left.iter().map(|s| s.content.chars().count()).sum();
    let width = area.width as usize;
    let gap_min = 2;

    if left_w + gap_min + right_w <= width {
        left.push(Span::raw(" ".repeat(width - left_w - right_w)));
        left.push(Span::styled(pid_text, Style::default().fg(theme.dim)));
    } else if left_w + gap_min <= width {
        let room = width - left_w - gap_min;
        let truncated: String = pid_text.chars().take(room).collect();
        left.push(Span::raw(" ".repeat(gap_min)));
        left.push(Span::styled(truncated, Style::default().fg(theme.dim)));
    } else {
        // left already too wide; drop the right metadata entirely
    }

    f.render_widget(Paragraph::new(Line::from(left)), area);
}

fn fmt_elapsed(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}
