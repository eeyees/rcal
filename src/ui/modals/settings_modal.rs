use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::model::WeekStartDay;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 66.min(area.width.saturating_sub(4));
    let popup_height = 16.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let lang = app.lang;
    let title = match lang {
        crate::i18n::Language::Ko => " ⚙ 환경 설정 (Settings) ",
        crate::i18n::Language::En => " ⚙ Settings & Preferences ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(title, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title 1: Week start day
            Constraint::Length(1), // Subtitle
            Constraint::Length(2), // 3 Buttons for Week start day
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Title 2: Language
            Constraint::Length(2), // 2 Buttons for Language
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Keyboard navigation guide
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Close button
        ])
        .split(inner);

    // Section 1: Week Start Day
    let sec1_title = match lang {
        crate::i18n::Language::Ko => "▶ 주간 시작 요일 (First Day of Week)",
        crate::i18n::Language::En => "▶ First Day of Week",
    };
    f.render_widget(
        Paragraph::new(Span::styled(sec1_title, Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD))),
        chunks[0],
    );

    let sec1_desc = match lang {
        crate::i18n::Language::Ko => "  달력(월간) 및 주간 타임라인의 첫 번째 요일을 지정합니다:",
        crate::i18n::Language::En => "  Choose which day starts the week in Month and Week views:",
    };
    f.render_widget(
        Paragraph::new(Span::styled(sec1_desc, Style::default().fg(Theme::TEXT_MUTED))),
        chunks[1],
    );

    let week_btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // Sunday
            Constraint::Length(2),
            Constraint::Length(18), // Monday
            Constraint::Length(2),
            Constraint::Length(18), // Saturday
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    let options = [
        (WeekStartDay::Sunday, "일요일 (Sun)", "[1]"),
        (WeekStartDay::Monday, "월요일 (Mon)", "[2]"),
        (WeekStartDay::Saturday, "토요일 (Sat)", "[3]"),
    ];

    for (idx, (day, label, shortcut)) in options.iter().enumerate() {
        let is_selected = app.week_start_day == *day;
        let prefix = if is_selected { "● " } else { "○ " };
        let text = format!("{}{} {}", prefix, label, shortcut);

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Theme::PRIMARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Theme::TEXT_NORMAL)
                .bg(Color::Rgb(40, 45, 55))
        };

        let rect = week_btn_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingWeekStart(*day));
    }

    // Section 2: Language
    let sec2_title = match lang {
        crate::i18n::Language::Ko => "▶ 기본 언어 (Language)",
        crate::i18n::Language::En => "▶ Display Language",
    };
    f.render_widget(
        Paragraph::new(Span::styled(sec2_title, Style::default().fg(Theme::SECONDARY).add_modifier(Modifier::BOLD))),
        chunks[4],
    );

    let lang_btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // 한국어
            Constraint::Length(2),
            Constraint::Length(20), // English
            Constraint::Min(0),
        ])
        .split(chunks[5]);

    let is_ko = app.lang == crate::i18n::Language::Ko;
    let ko_style = if is_ko {
        Style::default().fg(Color::Black).bg(Theme::PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_NORMAL).bg(Color::Rgb(40, 45, 55))
    };
    let ko_p = Paragraph::new(Line::from(vec![Span::raw(if is_ko { "● 한국어 [L]" } else { "○ 한국어 [L]" })]))
        .style(ko_style)
        .alignment(Alignment::Center);
    f.render_widget(ko_p, lang_btn_chunks[0]);
    app.hit_map.register(lang_btn_chunks[0], HitTarget::BtnLanguage);

    let en_style = if !is_ko {
        Style::default().fg(Color::Black).bg(Theme::PRIMARY).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Theme::TEXT_NORMAL).bg(Color::Rgb(40, 45, 55))
    };
    let en_p = Paragraph::new(Line::from(vec![Span::raw(if !is_ko { "● English [L]" } else { "○ English [L]" })]))
        .style(en_style)
        .alignment(Alignment::Center);
    f.render_widget(en_p, lang_btn_chunks[2]);
    app.hit_map.register(lang_btn_chunks[2], HitTarget::BtnLanguage);

    // Section 3: Keyboard shortcuts guide
    let hint_line = match lang {
        crate::i18n::Language::Ko => Line::from(vec![
            Span::styled("단축키: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("[1/2/3]", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" 요일선택  "),
            Span::styled("[Tab/←/→]", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" 순환이동  "),
            Span::styled("[L]", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" 언어전환"),
        ]),
        crate::i18n::Language::En => Line::from(vec![
            Span::styled("Shortcuts: ", Style::default().fg(Theme::TEXT_MUTED)),
            Span::styled("[1/2/3]", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" Select Day  "),
            Span::styled("[Tab/←/→]", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" Cycle  "),
            Span::styled("[L]", Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::raw(" Language"),
        ]),
    };
    f.render_widget(Paragraph::new(hint_line).alignment(Alignment::Center), chunks[7]);

    // Close button
    let close_text = match lang {
        crate::i18n::Language::Ko => "[ 닫기 (Esc / Enter) ]",
        crate::i18n::Language::En => "[ Close (Esc / Enter) ]",
    };
    let close_btn = Paragraph::new(Line::from(vec![Span::raw(close_text)]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(close_btn, chunks[9]);
    app.hit_map.register(chunks[9], HitTarget::ModalClose);
}
