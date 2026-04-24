//! Rich help modal: full keybinding reference with scrolling and
//! active-section highlighting. Content is a `const` table; the
//! section matching `Mode::Help::origin` is drawn cyan + bold.

use crate::app::help_section::SectionId;
use crate::app::types::Mode;
use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

pub struct Entry {
    pub keys: &'static str,
    pub desc: &'static str,
}

pub struct Section {
    pub id: SectionId,
    pub title: &'static str,
    pub entries: &'static [Entry],
}

pub const SECTIONS: &[Section] = &[
    Section {
        id: SectionId::ListFocus,
        title: "List focus",
        entries: &[
            Entry {
                keys: "j / k, ↓ / ↑",
                desc: "move cursor within recipes list",
            },
            Entry {
                keys: "h / l, ← / →",
                desc: "cycle through the recipe's previous run sessions",
            },
            Entry {
                keys: "Enter",
                desc: "run recipe — if a session is already running for it, jump there",
            },
            Entry {
                keys: "Shift+Enter / r",
                desc: "always spawn a new session (never reuse)",
            },
            Entry {
                keys: "/",
                desc: "enter fuzzy-filter mode",
            },
            Entry {
                keys: "d",
                desc: "open justfile-switcher dropdown",
            },
            Entry {
                keys: "Tab",
                desc: "cycle focus between list and session pane",
            },
            Entry {
                keys: "K",
                desc: "kill the focused session (confirms)",
            },
            Entry {
                keys: "x",
                desc: "close the focused session (confirms)",
            },
            Entry {
                keys: "Ctrl+o / Ctrl+i",
                desc: "jump to next / previous session with unread output",
            },
            Entry {
                keys: "L",
                desc: "copy the focused session's log path",
            },
            Entry {
                keys: "> / < / =",
                desc: "grow / shrink / reset the left pane width",
            },
            Entry {
                keys: "F1 / ?",
                desc: "open this help",
            },
            Entry {
                keys: "e",
                desc: "open the startup-errors modal",
            },
            Entry {
                keys: "q",
                desc: "quit — confirms if sessions are running",
            },
        ],
    },
    Section {
        id: SectionId::SessionFocus,
        title: "Session focus",
        entries: &[
            Entry {
                keys: "F12 / Ctrl+g",
                desc: "return focus to the recipes list",
            },
            Entry {
                keys: "PgUp / PgDn",
                desc: "scroll session output up / down",
            },
            Entry {
                keys: "Home / End",
                desc: "jump to top / bottom of scrollback",
            },
            Entry {
                keys: "(all other keys)",
                desc: "forwarded to the running shell as typed input",
            },
            Entry {
                keys: "F1",
                desc: "open help (globally intercepted; does not reach the shell)",
            },
        ],
    },
    Section {
        id: SectionId::Filter,
        title: "Filter mode (after /)",
        entries: &[
            Entry {
                keys: "a–z, 0–9, …",
                desc: "extend the filter pattern",
            },
            Entry {
                keys: "Backspace",
                desc: "remove last character",
            },
            Entry {
                keys: "Enter",
                desc: "commit filter and return to list",
            },
            Entry {
                keys: "Esc",
                desc: "discard filter and return to list",
            },
        ],
    },
    Section {
        id: SectionId::Dropdown,
        title: "Justfile dropdown (after d)",
        entries: &[
            Entry {
                keys: "a–z, …",
                desc: "filter the justfile list",
            },
            Entry {
                keys: "j / k, ↑ / ↓",
                desc: "move cursor",
            },
            Entry {
                keys: "Enter",
                desc: "select justfile",
            },
            Entry {
                keys: "Esc",
                desc: "cancel",
            },
        ],
    },
    Section {
        id: SectionId::Param,
        title: "Param input (modal)",
        entries: &[
            Entry {
                keys: "a–z, 0–9, …",
                desc: "edit the current parameter",
            },
            Entry {
                keys: "Backspace",
                desc: "remove last character",
            },
            Entry {
                keys: "Tab",
                desc: "move to next parameter",
            },
            Entry {
                keys: "Enter",
                desc: "commit all parameters and spawn the recipe",
            },
            Entry {
                keys: "Esc",
                desc: "cancel",
            },
        ],
    },
    Section {
        id: SectionId::Confirm,
        title: "Confirm prompt (K / x / q on running sessions)",
        entries: &[
            Entry {
                keys: "y / Enter",
                desc: "confirm",
            },
            Entry {
                keys: "n / c / Esc",
                desc: "cancel",
            },
        ],
    },
    Section {
        id: SectionId::Errors,
        title: "Errors list (after e)",
        entries: &[Entry {
            keys: "Esc / q / e",
            desc: "close",
        }],
    },
    Section {
        id: SectionId::HelpItself,
        title: "Help (this modal)",
        entries: &[
            Entry {
                keys: "j / k, ↑ / ↓",
                desc: "scroll by one line",
            },
            Entry {
                keys: "PgUp / PgDn",
                desc: "scroll by ten lines",
            },
            Entry {
                keys: "Home / End",
                desc: "jump to top / bottom",
            },
            Entry {
                keys: "Esc / q / ? / F1",
                desc: "close",
            },
        ],
    },
];

fn build_lines(origin: SectionId, theme: &crate::theme::Theme) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    for (idx, section) in SECTIONS.iter().enumerate() {
        if idx > 0 {
            out.push(Line::from(""));
        }
        let is_active = section.id == origin;
        let marker = if is_active { "▸ " } else { "  " };
        let title_style = if is_active {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };
        out.push(Line::from(vec![
            Span::raw(marker),
            Span::styled(section.title, title_style),
        ]));
        for e in section.entries {
            out.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{:<20}", e.keys), Style::default().fg(theme.fg)),
                Span::raw("  "),
                Span::raw(e.desc),
            ]));
        }
    }
    out
}

pub fn render(f: &mut Frame, app: &App, area: Rect, theme: &crate::theme::Theme) {
    let (scroll, origin) = match &app.mode {
        Mode::Help { scroll, origin } => (*scroll, *origin),
        _ => return,
    };
    let lines = build_lines(origin, theme);
    let inner_rows = area.height.saturating_sub(2);
    let max_scroll = (lines.len() as u16).saturating_sub(inner_rows);
    let clamped = scroll.min(max_scroll);
    let para = Paragraph::new(lines)
        .block(crate::ui::modal_base::block("help", theme))
        .wrap(Wrap { trim: false })
        .scroll((clamped, 0));
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_section_id_has_a_section() {
        let all = [
            SectionId::ListFocus,
            SectionId::SessionFocus,
            SectionId::Filter,
            SectionId::Dropdown,
            SectionId::Param,
            SectionId::Confirm,
            SectionId::Errors,
            SectionId::HelpItself,
        ];
        for id in all {
            assert!(
                SECTIONS.iter().any(|s| s.id == id),
                "missing section for {id:?}"
            );
        }
    }

    #[test]
    fn build_lines_marks_only_the_origin_section() {
        let theme = crate::theme::registry::resolve(crate::theme::DEFAULT_THEME_NAME);
        let lines = build_lines(SectionId::Filter, &theme);
        let active_count = lines
            .iter()
            .filter(|l| {
                l.spans
                    .first()
                    .map(|s| s.content.as_ref() == "▸ ")
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(active_count, 1);
    }
}
