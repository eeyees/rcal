use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(28),
        ])
        .split(area);

    let lang = app.lang;
    let status_line = if let Some(msg) = app.get_active_status() {
        Line::from(vec![
            Span::styled(" ℹ ", Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(msg, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)),
        ])
    } else {
        match lang {
            crate::i18n::Language::Ko => Line::from(vec![
                Span::styled(" 단축키: ", Style::default().fg(Theme::TEXT_MUTED)),
                Span::styled("1-4", Style::default().fg(Theme::PRIMARY)),
                Span::raw(" 뷰 | "),
                Span::styled("c/n", Style::default().fg(Theme::SECONDARY)),
                Span::raw(" 새 일정 | "),
                Span::styled("r", Style::default().fg(Theme::ACCENT)),
                Span::raw(" 동기화 | "),
                Span::styled("L", Style::default().fg(Theme::PRIMARY)),
                Span::raw(" 언어 | "),
                Span::styled("A", Style::default().fg(Theme::SECONDARY)),
                Span::raw(" 계정 | "),
                Span::styled("?", Style::default().fg(Theme::TEXT_MUTED)),
                Span::raw(" 도움말 | "),
                Span::styled("q", Style::default().fg(Theme::DANGER)),
                Span::raw(" 종료"),
            ]),
            crate::i18n::Language::En => Line::from(vec![
                Span::styled(" Keys: ", Style::default().fg(Theme::TEXT_MUTED)),
                Span::styled("1-4", Style::default().fg(Theme::PRIMARY)),
                Span::raw(" View | "),
                Span::styled("c/n", Style::default().fg(Theme::SECONDARY)),
                Span::raw(" New | "),
                Span::styled("r", Style::default().fg(Theme::ACCENT)),
                Span::raw(" Sync | "),
                Span::styled("L", Style::default().fg(Theme::PRIMARY)),
                Span::raw(" Lang | "),
                Span::styled("A", Style::default().fg(Theme::SECONDARY)),
                Span::raw(" Accounts | "),
                Span::styled("?", Style::default().fg(Theme::TEXT_MUTED)),
                Span::raw(" Help | "),
                Span::styled("q", Style::default().fg(Theme::DANGER)),
                Span::raw(" Quit"),
            ]),
        }
    };

    let p_left = Paragraph::new(status_line).style(Style::default().bg(Color::Rgb(25, 28, 36)));
    f.render_widget(p_left, chunks[0]);

    let now_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let p_right = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {} ", now_str), Style::default().fg(Theme::TEXT_DIM)),
    ]))
    .alignment(Alignment::Right)
    .style(Style::default().bg(Color::Rgb(25, 28, 36)));
    f.render_widget(p_right, chunks[1]);
}
