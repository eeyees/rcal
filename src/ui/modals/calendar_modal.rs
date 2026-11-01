use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, CalendarModalState};
use crate::model::Calendar;
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, state: &CalendarModalState, area: Rect) {
    let popup_width = 72.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);
    app.hit_map.register(popup_rect, HitTarget::ModalConsume);

    let lang = app.lang;
    let theme = &app.theme;
    let title = if state.is_creating {
        lang.t().modal_cal_create_title
    } else {
        lang.t().modal_cal_manage_title
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    if state.is_creating {
        render_create_form(f, app, state, inner);
    } else {
        render_calendar_list(f, app, state, inner);
    }
}

fn render_create_form(f: &mut Frame, app: &mut App, state: &CalendarModalState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name input
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Color selector
            Constraint::Min(2),    // Notes
            Constraint::Length(1), // Action Buttons
        ])
        .split(area);

    let lang = app.lang;
    let theme = &app.theme;
    let name_label = lang.t().cal_name_label;
    let placeholder = lang.t().cal_name_placeholder;

    let is_name_active = state.active_field == 0;
    let name_style = if is_name_active {
        Style::default()
            .fg(theme.text_highlight)
            .bg(theme.selected_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_normal)
    };

    let name_line = Line::from(vec![
        Span::styled(name_label, Style::default().fg(theme.text_muted)),
        Span::styled(
            if state.name.is_empty() {
                placeholder
            } else {
                &state.name
            },
            name_style,
        ),
    ]);
    f.render_widget(Paragraph::new(name_line), chunks[0]);

    // Color Selector
    let color_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(8), // Blue
            Constraint::Length(1),
            Constraint::Length(8), // Green
            Constraint::Length(1),
            Constraint::Length(8), // Yellow
            Constraint::Length(1),
            Constraint::Length(8), // Red
            Constraint::Length(1),
            Constraint::Length(8), // Purple
            Constraint::Length(1),
            Constraint::Length(8), // Cyan
            Constraint::Length(1),
            Constraint::Length(8), // Orange
            Constraint::Min(0),
        ])
        .split(chunks[2]);

    let color_names = lang.t().cal_colors;

    for i in 0..7 {
        let is_selected = state.color_idx == i;
        let c_code = CalendarModalState::PALETTE[i];
        let rat_color = Calendar {
            id: String::new(),
            account_id: None,
            name: String::new(),
            color_code: c_code.to_string(),
            provider: crate::model::ProviderType::Local,
            enabled: true,
            is_default: false,
            remote_url: None,
            ctag: None,
        }
        .ratatui_color();

        let prefix = if is_selected { "▶● " } else { " ● " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(rat_color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(rat_color).bg(Color::Rgb(30, 35, 45))
        };

        let rect = color_chunks[i * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(format!(
            "{}{}",
            prefix, color_names[i]
        ))]))
        .style(style)
        .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalCalColor(i));
    }

    // Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(14), // [ 저장 ]
            Constraint::Length(3),
            Constraint::Length(14), // [ 취소 ]
            Constraint::Min(1),
        ])
        .split(chunks[4]);

    let is_save_focused = state.active_field == 2;
    let save_text = if is_save_focused {
        format!(" ▶ {} ◀ ", lang.btn_save())
    } else {
        format!(" {} ", lang.btn_save())
    };
    let save_style = if is_save_focused {
        Style::default()
            .fg(Color::Black)
            .bg(theme.text_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(theme.secondary)
            .add_modifier(Modifier::BOLD)
    };
    let btn_save = Paragraph::new(Line::from(vec![Span::raw(save_text)]))
        .style(save_style)
        .alignment(Alignment::Center);
    f.render_widget(btn_save, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalCalSave);

    let is_cancel_focused = state.active_field == 3;
    let cancel_text = if is_cancel_focused {
        format!(" ▶ {} ◀ ", lang.btn_cancel())
    } else {
        format!(" {} ", lang.btn_cancel())
    };
    let cancel_style = if is_cancel_focused {
        Style::default()
            .fg(Color::Black)
            .bg(theme.text_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        theme.button_style()
    };
    let btn_cancel = Paragraph::new(Line::from(vec![Span::raw(cancel_text)]))
        .style(cancel_style)
        .alignment(Alignment::Center);
    f.render_widget(btn_cancel, btn_chunks[3]);
    app.hit_map.register(btn_chunks[3], HitTarget::ModalCancel);

    // Terminal cursor positioning on active input field
    let cursor_pos = match state.active_field {
        0 => {
            let label_w = Span::raw(name_label).width() as u16;
            let val_w = Span::raw(&state.name).width() as u16;
            (chunks[0].x + label_w + val_w, chunks[0].y)
        }
        1 => {
            let rect = color_chunks[(state.color_idx % 7) * 2];
            (rect.x + rect.width / 2, rect.y)
        }
        2 => (btn_chunks[1].x + btn_chunks[1].width / 2, btn_chunks[1].y),
        3 => (btn_chunks[3].x + btn_chunks[3].width / 2, btn_chunks[3].y),
        _ => (chunks[0].x, chunks[0].y),
    };
    let safe_x = cursor_pos.0.min(area.right().saturating_sub(2));
    let safe_y = cursor_pos.1.min(area.bottom().saturating_sub(1));
    f.set_cursor_position((safe_x, safe_y));
}

fn render_calendar_list(f: &mut Frame, app: &mut App, state: &CalendarModalState, area: Rect) {
    let calendars = app.calendars.clone();
    let lang = app.lang;
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Spacer
            Constraint::Min(6),    // List of calendars
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Action Buttons
        ])
        .split(area);

    let header_text = lang.t().cal_list_header;
    f.render_widget(
        Paragraph::new(Span::styled(
            header_text,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        )),
        chunks[0],
    );

    for (idx, cal) in calendars.iter().enumerate() {
        let row_y = chunks[2].y + idx as u16;
        if row_y >= chunks[2].y + chunks[2].height {
            break;
        }

        let row_rect = Rect {
            x: chunks[2].x,
            y: row_y,
            width: chunks[2].width,
            height: 1,
        };

        let row_parts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(30),    // Left: Calendar info
                Constraint::Length(12), // Right: [ 삭제 ] button
            ])
            .split(row_rect);

        let cal_color = cal.ratatui_color();
        let status_text = if cal.enabled {
            lang.t().cal_status_visible
        } else {
            lang.t().cal_status_hidden
        };

        let is_selected = idx == state.selected_calendar_idx;
        let bullet = if is_selected { " ▶ " } else { " ● " };
        let bullet_color = if is_selected { theme.accent } else { cal_color };

        let info_line = Line::from(vec![
            Span::styled(bullet, Style::default().fg(bullet_color)),
            Span::styled(
                &cal.name,
                Style::default()
                    .fg(if is_selected {
                        theme.text_highlight
                    } else {
                        theme.text_normal
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({}) ", cal.provider.as_str()),
                Style::default().fg(theme.text_dim),
            ),
            Span::styled(
                status_text,
                Style::default().fg(if cal.enabled {
                    theme.secondary
                } else {
                    theme.text_dim
                }),
            ),
        ]);
        let row_style = if is_selected {
            Style::default().bg(theme.selected_bg)
        } else {
            Style::default()
        };
        f.render_widget(Paragraph::new(info_line).style(row_style), row_parts[0]);

        if !cal.is_default {
            let del_label = lang.btn_delete();
            let del_btn = Paragraph::new(Line::from(vec![Span::raw(del_label)]))
                .style(
                    Style::default()
                        .fg(theme.danger)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            f.render_widget(del_btn, row_parts[1]);
            app.hit_map
                .register(row_parts[1], HitTarget::ModalCalDelete(cal.id.clone()));
        } else {
            let def_label = lang.t().cal_default_badge;
            let def_text = Paragraph::new(Line::from(vec![Span::raw(def_label)]))
                .style(Style::default().fg(theme.text_dim))
                .alignment(Alignment::Center);
            f.render_widget(def_text, row_parts[1]);
        }
    }

    // Bottom Action Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // [ + 새 캘린더 만들기 ]
            Constraint::Min(1),
            Constraint::Length(14), // [ 닫기 ]
        ])
        .split(chunks[4]);

    let add_label = lang.t().cal_btn_create;
    let btn_add = Paragraph::new(Line::from(vec![Span::raw(add_label)]))
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_add, btn_chunks[0]);
    app.hit_map.register(btn_chunks[0], HitTarget::ModalCalNew);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::ModalClose);

    if calendars.is_empty() {
        f.set_cursor_position((btn_chunks[0].x + btn_chunks[0].width / 2, btn_chunks[0].y));
    } else {
        let sel_idx = state
            .selected_calendar_idx
            .min(calendars.len().saturating_sub(1));
        let cursor_y = chunks[2].y + sel_idx as u16;
        f.set_cursor_position((chunks[2].x + 1, cursor_y));
    }
}
