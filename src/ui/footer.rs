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

    let t = lang.t();
    let nav_label = match lang {
        crate::i18n::Language::Ko => "탐색",
        crate::i18n::Language::Ja => "移動",
        _ => "Navigate",
    };
    let (settings_label, theme_label) = match lang {
        crate::i18n::Language::Ko => ("설정", "테마"),
        crate::i18n::Language::Ja => ("設定", "テーマ"),
        _ => ("Setup", "Theme"),
    };
    let key = |k: &'static str, color| Span::styled(k, Style::default().fg(color));
    let status_line = Line::from(vec![
        Span::styled(
            format!(" {} ", t.footer_keys_label),
            Style::default().fg(theme.text_muted),
        ),
        key("1-4", theme.primary),
        Span::raw(format!(" {} | ", t.footer_view)),
        key("b", theme.primary),
        Span::raw(format!(" {} | ", t.footer_sidebar)),
        key("h/j/k/l", theme.accent),
        Span::raw(format!(" {} | ", nav_label)),
        key("t", theme.secondary),
        Span::raw(format!(" {} | ", t.btn_today)),
        key("n", theme.secondary),
        Span::raw(format!(" {} | ", t.footer_new)),
        key("r", theme.primary),
        Span::raw(format!(" {} | ", t.footer_sync)),
        key("A", theme.primary),
        Span::raw(format!(" {} | ", t.footer_accounts)),
        key("s", theme.primary),
        Span::raw(format!(" {} | ", settings_label)),
        key("T", theme.primary),
        Span::raw(format!(" {} | ", theme_label)),
        key("L", theme.primary),
        Span::raw(format!(" {} | ", t.footer_lang)),
        key("?", theme.text_muted),
        Span::raw(format!(" {} | ", t.footer_help)),
        key("q", theme.danger),
        Span::raw(format!(" {}", t.footer_quit)),
    ]);

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

/// Message line (shown where the action-button row used to be).
pub fn render_message(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    if let Some(msg) = app.get_active_status() {
        let line = Line::from(vec![
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
        ]);
        f.render_widget(Paragraph::new(line), area);
    }
}
