use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(28)])
        .split(area);

    let lang = app.lang;
    let theme = &app.theme;

    let status_line = if let Some(msg) = app.get_active_status() {
        Line::from(vec![
            Span::styled(
                " ℹ ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                msg,
                Style::default()
                    .fg(theme.text_highlight)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        let t = lang.t();
        let nav_label = match lang {
            crate::i18n::Language::Ko => "탐색",
            crate::i18n::Language::Ja => "移動",
            _ => "Navigate",
        };
        Line::from(vec![
            Span::styled(
                format!(" {} ", t.footer_keys_label),
                Style::default().fg(theme.text_muted),
            ),
            Span::styled("1-4", Style::default().fg(theme.primary)),
            Span::raw(format!(" {} | ", t.footer_view)),
            Span::styled("b", Style::default().fg(theme.primary)),
            Span::raw(format!(" {} | ", t.footer_sidebar)),
            Span::styled("h/j/k/l", Style::default().fg(theme.accent)),
            Span::raw(format!(" {} | ", nav_label)),
            Span::styled("t", Style::default().fg(theme.secondary)),
            Span::raw(format!(" {} | ", t.btn_today)),
            Span::styled("?", Style::default().fg(theme.text_muted)),
            Span::raw(format!(" {} | ", t.footer_help)),
            Span::styled("q", Style::default().fg(theme.danger)),
            Span::raw(format!(" {}", t.footer_quit)),
        ])
    };

    let p_left = Paragraph::new(status_line).style(Style::default().bg(theme.panel_bg));
    f.render_widget(p_left, chunks[0]);

    let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let p_right = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {} ", now_str),
        Style::default().fg(theme.text_dim),
    )]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(theme.panel_bg));
    f.render_widget(p_right, chunks[1]);
}
