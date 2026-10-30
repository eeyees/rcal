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
use crate::ui::theme::Theme;

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

    let lang = app.lang;
    let title = if state.is_creating {
        match lang {
            crate::i18n::Language::Ko => " 새 캘린더 생성 (카테고리 / 분류) ",
            crate::i18n::Language::En => " Create New Calendar ",
        }
    } else {
        match lang {
            crate::i18n::Language::Ko => " 캘린더 관리 & 카테고리 설정 ",
            crate::i18n::Language::En => " Manage Calendars & Categories ",
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(title, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    if state.is_creating {
        render_create_form(f, app, state, inner);
    } else {
        render_calendar_list(f, app, inner);
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
    let name_label = match lang {
        crate::i18n::Language::Ko => " 캘린더 이름: ",
        crate::i18n::Language::En => " Name:          ",
    };
    let placeholder = match lang {
        crate::i18n::Language::Ko => "(예: 스터디, 운동, 가족, 사이드 프로젝트)",
        crate::i18n::Language::En => "(e.g. Study, Fitness, Family, Side Project)",
    };

    let name_line = Line::from(vec![
        Span::styled(name_label, Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.name.is_empty() { placeholder } else { &state.name },
            Style::default().fg(Theme::TEXT_HIGHLIGHT).bg(Color::Rgb(40, 60, 100)).add_modifier(Modifier::BOLD),
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

    let color_names = match lang {
        crate::i18n::Language::Ko => ["파랑", "초록", "노랑", "빨강", "보라", "청록", "주황"],
        crate::i18n::Language::En => ["Blue", "Green", "Yellow", "Red", "Purple", "Cyan", "Orange"],
    };

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
        }.ratatui_color();

        let prefix = if is_selected { "▶● " } else { " ● " };
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(rat_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(rat_color).bg(Color::Rgb(30, 35, 45))
        };

        let rect = color_chunks[i * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(format!("{}{}", prefix, color_names[i]))]))
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

    let btn_save = Paragraph::new(Line::from(vec![Span::raw(lang.btn_save())]))
        .style(Style::default().fg(Color::Black).bg(Theme::SECONDARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_save, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalCalSave);

    let btn_cancel = Paragraph::new(Line::from(vec![Span::raw(lang.btn_cancel())]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_cancel, btn_chunks[3]);
    app.hit_map.register(btn_chunks[3], HitTarget::ModalCancel);
}

fn render_calendar_list(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // List of calendars
            Constraint::Length(1), // Action Buttons
        ])
        .split(area);

    let lang = app.lang;
    let mut lines = Vec::new();
    let header_text = match lang {
        crate::i18n::Language::Ko => "등록된 캘린더 목록 (일정 등록 시 선택하여 분류 가능):",
        crate::i18n::Language::En => "Registered Calendars (Select when creating events):",
    };
    lines.push(Line::from(Span::styled(header_text, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD))));

    for (idx, cal) in app.calendars.iter().enumerate() {
        let cal_color = cal.ratatui_color();
        let status_text = if cal.enabled {
            match lang {
                crate::i18n::Language::Ko => "[표시중]",
                crate::i18n::Language::En => "[Visible]",
            }
        } else {
            match lang {
                crate::i18n::Language::Ko => "[숨김]",
                crate::i18n::Language::En => "[Hidden]",
            }
        };

        let del_label = match lang {
            crate::i18n::Language::Ko => "[ 삭제 ]",
            crate::i18n::Language::En => "[ Delete ]",
        };

        let line = Line::from(vec![
            Span::styled(" ● ", Style::default().fg(cal_color)),
            Span::styled(&cal.name, Style::default().fg(Theme::TEXT_NORMAL).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({}) ", cal.provider.as_str()), Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(status_text, Style::default().fg(if cal.enabled { Theme::SECONDARY } else { Theme::TEXT_DIM })),
            Span::raw("  "),
            if !cal.is_default {
                Span::styled(del_label, Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(match lang {
                    crate::i18n::Language::Ko => "(기본)",
                    crate::i18n::Language::En => "(Default)",
                }, Style::default().fg(Theme::TEXT_DIM))
            },
        ]);
        lines.push(line);

        // Register delete button if not default
        if !cal.is_default {
            let row_y = chunks[0].y + 1 + idx as u16;
            if row_y < chunks[0].y + chunks[0].height {
                let del_rect = Rect {
                    x: chunks[0].x + chunks[0].width.saturating_sub(14),
                    y: row_y,
                    width: 12,
                    height: 1,
                };
                app.hit_map.register(del_rect, HitTarget::ModalCalDelete(cal.id.clone()));
            }
        }
    }

    f.render_widget(Paragraph::new(lines), chunks[0]);

    // Bottom Action Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // [ + 새 캘린더 만들기 ]
            Constraint::Min(1),
            Constraint::Length(14), // [ 닫기 ]
        ])
        .split(chunks[1]);

    let add_label = match lang {
        crate::i18n::Language::Ko => " [ + 새 캘린더 생성 ] ",
        crate::i18n::Language::En => " [ + Create Calendar ] ",
    };
    let btn_add = Paragraph::new(Line::from(vec![Span::raw(add_label)]))
        .style(Style::default().fg(Color::Black).bg(Theme::PRIMARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_add, btn_chunks[0]);
    app.hit_map.register(btn_chunks[0], HitTarget::ModalCalNew);

    let btn_close = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::ModalClose);
}
