use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(app.lang.sidebar_calendars());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.calendars.len() as u16 + 4), // Calendar list + manage button
            Constraint::Length(1),                              // Divider
            Constraint::Min(6),                                 // Shortcuts hint
        ])
        .split(inner);

    // 1. Render Calendar items with clickable checkbox
    let mut cal_lines = Vec::new();
    for (idx, cal) in app.calendars.iter().enumerate() {
        let check_box = if cal.enabled { "[x] " } else { "[ ] " };
        let check_color = if cal.enabled { Theme::SECONDARY } else { Theme::TEXT_DIM };
        let cal_color = cal.ratatui_color();

        let max_name_len = (chunks[0].width as usize).saturating_sub(16);
        let display_name = if cal.name.chars().count() > max_name_len && max_name_len > 3 {
            let truncated: String = cal.name.chars().take(max_name_len.saturating_sub(2)).collect();
            format!("{}..", truncated)
        } else {
            cal.name.clone()
        };

        let line = Line::from(vec![
            Span::styled(check_box, Style::default().fg(check_color).add_modifier(Modifier::BOLD)),
            Span::styled("● ", Style::default().fg(cal_color)),
            Span::styled(display_name, Style::default().fg(if cal.enabled { Theme::TEXT_NORMAL } else { Theme::TEXT_MUTED })),
            Span::styled(
                format!(" ({})", cal.provider.as_str()),
                Style::default().fg(Theme::TEXT_DIM),
            ),
        ]);
        cal_lines.push(line);

        // Register hit region for this calendar row
        if (idx as u16) < chunks[0].height {
            let row_rect = Rect {
                x: chunks[0].x,
                y: chunks[0].y + idx as u16,
                width: chunks[0].width,
                height: 1,
            };
            app.hit_map.register(row_rect, HitTarget::CalendarToggle(cal.id.clone()));
        }
    }

    // Add "[ + 관리/추가 (C) ]" button under calendars
    let manage_label = match app.lang {
        crate::i18n::Language::Ko => " + 캘린더 관리/추가 (C) ",
        crate::i18n::Language::En => " + Manage Calendars (C) ",
    };
    cal_lines.push(Line::from(""));
    cal_lines.push(Line::from(vec![
        Span::styled(
            manage_label,
            Style::default().fg(Theme::PRIMARY).add_modifier(Modifier::BOLD),
        ),
    ]));

    let btn_rect = Rect {
        x: chunks[0].x,
        y: chunks[0].y + app.calendars.len() as u16 + 1,
        width: chunks[0].width,
        height: 1,
    };
    app.hit_map.register(btn_rect, HitTarget::BtnManageCalendars);

    f.render_widget(Paragraph::new(cal_lines), chunks[0]);

    // 2. Localized Quick Tips in sidebar
    let mut tips = Vec::new();
    tips.push(Line::from(Span::styled(app.lang.sidebar_quick_nav(), Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))));
    for t in app.lang.sidebar_tips() {
        tips.push(Line::from(Span::styled(t, Style::default().fg(Theme::TEXT_MUTED))));
    }

    tips.push(Line::from(""));
    tips.push(Line::from(Span::styled(app.lang.sidebar_keys(), Style::default().fg(Theme::ACCENT).add_modifier(Modifier::BOLD))));
    for k in app.lang.sidebar_key_tips() {
        tips.push(Line::from(Span::styled(k, Style::default().fg(Theme::TEXT_MUTED))));
    }

    f.render_widget(Paragraph::new(tips), chunks[2]);
}
