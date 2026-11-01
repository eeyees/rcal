use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(app.lang.sidebar_calendars());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(app.calendars.len() as u16 + 4), // Calendar list + manage button
            Constraint::Length(1),                              // Divider
            Constraint::Min(4),                                 // Shortcuts hint
            Constraint::Length(1),                              // Hide sidebar button
        ])
        .split(inner);

    // 1. Render Calendar items with clickable checkbox
    let mut cal_lines = Vec::new();
    for (idx, cal) in app.calendars.iter().enumerate() {
        let check_box = if cal.enabled { "[x] " } else { "[ ] " };
        let check_color = if cal.enabled { theme.secondary } else { theme.text_dim };
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
            Span::styled(display_name, Style::default().fg(if cal.enabled { theme.text_normal } else { theme.text_muted })),
            Span::styled(
                format!(" ({})", cal.provider.as_str()),
                Style::default().fg(theme.text_dim),
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
    let manage_label = app.lang.sidebar_manage();
    cal_lines.push(Line::from(""));
    cal_lines.push(Line::from(vec![
        Span::styled(
            manage_label,
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
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
    tips.push(Line::from(Span::styled(app.lang.sidebar_quick_nav(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))));
    for t in app.lang.sidebar_tips() {
        tips.push(Line::from(Span::styled(t, Style::default().fg(theme.text_muted))));
    }

    tips.push(Line::from(""));
    tips.push(Line::from(Span::styled(app.lang.sidebar_keys(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))));
    for k in app.lang.sidebar_key_tips() {
        tips.push(Line::from(Span::styled(k, Style::default().fg(theme.text_muted))));
    }

    f.render_widget(Paragraph::new(tips), chunks[2]);

    // 3. Hide Sidebar Button / Shortcut Hint
    let hide_label = app.lang.sidebar_hide();
    let hide_btn = Paragraph::new(Line::from(vec![
        Span::styled(
            hide_label,
            Style::default().fg(theme.text_muted),
        ),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hide_btn, chunks[3]);
    app.hit_map.register(chunks[3], HitTarget::BtnToggleSidebar);
}
