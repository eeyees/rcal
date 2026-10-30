use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, EventFormState};
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, form: &EventFormState, area: Rect) {
    let popup_width = 66.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let lang = app.lang;
    let title_text = if form.is_new {
        lang.modal_new_event()
    } else {
        lang.modal_edit_event()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(title_text, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title input
            Constraint::Length(1), // Calendar select
            Constraint::Length(1), // Date (YYYY-MM-DD)
            Constraint::Length(1), // Start Time
            Constraint::Length(1), // End Time
            Constraint::Length(1), // All-day toggle
            Constraint::Length(1), // Location
            Constraint::Length(2), // Description
            Constraint::Min(1),    // Spacing
            Constraint::Length(1), // Action buttons [ Save ] [ Cancel ]
        ])
        .split(inner);

    let active_style = Style::default().fg(Theme::TEXT_HIGHLIGHT).bg(Color::Rgb(40, 60, 100)).add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Theme::TEXT_NORMAL);

    // 0: Title
    let title_line = Line::from(vec![
        Span::styled(lang.field_title(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if form.title.is_empty() { lang.placeholder_title() } else { &form.title },
            if form.active_field == 0 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);
    app.hit_map.register(chunks[0], HitTarget::ModalField(0));

    // 1: Calendar
    let cal_name = app.calendars.iter()
        .find(|c| c.id == form.calendar_id)
        .map(|c| c.name.as_str())
        .unwrap_or("Default");
    let cal_hint = match lang {
        crate::i18n::Language::Ko => "(스페이스바로 변경)",
        crate::i18n::Language::En => "(Space to cycle)",
    };
    let cal_line = Line::from(vec![
        Span::styled(lang.field_calendar(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            format!("[ {} ] {}", cal_name, cal_hint),
            if form.active_field == 1 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(cal_line), chunks[1]);
    app.hit_map.register(chunks[1], HitTarget::ModalField(1));

    // 2: Date
    let date_line = Line::from(vec![
        Span::styled(lang.field_date(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            &form.date_str,
            if form.active_field == 2 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(date_line), chunks[2]);
    app.hit_map.register(chunks[2], HitTarget::ModalField(2));

    // 3: Start Time
    let start_line = Line::from(vec![
        Span::styled(lang.field_start_time(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            &form.start_time_str,
            if form.active_field == 3 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(start_line), chunks[3]);
    app.hit_map.register(chunks[3], HitTarget::ModalField(3));

    // 4: End Time
    let end_line = Line::from(vec![
        Span::styled(lang.field_end_time(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            &form.end_time_str,
            if form.active_field == 4 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(end_line), chunks[4]);
    app.hit_map.register(chunks[4], HitTarget::ModalField(4));

    // 5: All Day
    let all_day_str = if form.all_day {
        match lang {
            crate::i18n::Language::Ko => "[x] 예",
            crate::i18n::Language::En => "[x] Yes",
        }
    } else {
        match lang {
            crate::i18n::Language::Ko => "[ ] 아니오",
            crate::i18n::Language::En => "[ ] No",
        }
    };
    let all_day_line = Line::from(vec![
        Span::styled(lang.field_all_day(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            all_day_str,
            if form.active_field == 5 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(all_day_line), chunks[5]);
    app.hit_map.register(chunks[5], HitTarget::ModalField(5));

    // 6: Location
    let loc_line = Line::from(vec![
        Span::styled(lang.field_location(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if form.location.is_empty() { lang.placeholder_location() } else { &form.location },
            if form.active_field == 6 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(loc_line), chunks[6]);
    app.hit_map.register(chunks[6], HitTarget::ModalField(6));

    // 7: Description
    let desc_line = Line::from(vec![
        Span::styled(lang.field_description(), Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if form.description.is_empty() { lang.placeholder_description() } else { &form.description },
            if form.active_field == 7 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(desc_line), chunks[7]);
    app.hit_map.register(chunks[7], HitTarget::ModalField(7));

    // Action Buttons: [ Save ] [ Cancel ]
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(14), // [ Save ]
            Constraint::Length(3),
            Constraint::Length(14), // [ Cancel ]
            Constraint::Min(1),
        ])
        .split(chunks[9]);

    let btn_save = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_save()))]))
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Theme::SECONDARY)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_save, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalSave);

    let btn_cancel = Paragraph::new(Line::from(vec![Span::raw(format!(" {} ", lang.btn_cancel()))]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_cancel, btn_chunks[3]);
    app.hit_map.register(btn_chunks[3], HitTarget::ModalCancel);
}
