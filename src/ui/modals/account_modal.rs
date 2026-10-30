use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{AccountModalState, App};
use crate::mouse::HitTarget;
use crate::ui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let popup_width = 78.min(area.width.saturating_sub(4));
    let popup_height = 22.min(area.height.saturating_sub(4));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);

    let lang = app.lang;
    let title = if state.is_adding {
        match lang {
            crate::i18n::Language::Ko => " 캘린더 계정 연동 (Google / MS 365 / Samsung / Naver / Apple) ",
            crate::i18n::Language::En => " Add Calendar Account (Google / MS 365 / Samsung / Naver / Apple) ",
        }
    } else {
        match lang {
            crate::i18n::Language::Ko => " 캘린더 계정 & 클라우드 실시간 동기화 ",
            crate::i18n::Language::En => " Calendar Accounts & Multi-Cloud Sync ",
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::PRIMARY))
        .title(Span::styled(title, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    if state.is_adding {
        render_add_form(f, app, state, inner);
    } else {
        render_account_list(f, app, state, inner);
    }
}

fn render_add_form(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Preset buttons row
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Name
            Constraint::Length(1), // URL / Endpoint
            Constraint::Length(1), // Username / ID
            Constraint::Length(1), // Password / OAuth Token
            Constraint::Min(2),    // Help notes
            Constraint::Length(1), // Action Buttons
        ])
        .split(area);

    // 6 Provider Presets: Naver, Apple, Google, Microsoft, Samsung, Generic CalDAV
    let preset_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11), // [ Naver ]
            Constraint::Length(1),
            Constraint::Length(11), // [ Apple ]
            Constraint::Length(1),
            Constraint::Length(11), // [ Google ]
            Constraint::Length(1),
            Constraint::Length(11), // [ MS 365 ]
            Constraint::Length(1),
            Constraint::Length(12), // [ Samsung ]
            Constraint::Length(1),
            Constraint::Length(11), // [ CalDAV ]
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let presets = [
        (0, " Naver ", preset_chunks[0]),
        (1, " Apple ", preset_chunks[2]),
        (2, " Google ", preset_chunks[4]),
        (3, " MS 365 ", preset_chunks[6]),
        (4, " Samsung ", preset_chunks[8]),
        (5, " CalDAV ", preset_chunks[10]),
    ];

    for (idx, label, rect) in presets {
        let is_selected = state.provider_idx == idx;
        let style = if is_selected {
            Theme::header_tab_active()
        } else {
            Theme::header_tab_inactive()
        };
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalAccountPreset(idx));
    }

    let active_style = Style::default().fg(Theme::TEXT_HIGHLIGHT).bg(Color::Rgb(40, 60, 100)).add_modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Theme::TEXT_NORMAL);

    let is_oauth = state.provider_idx == 2 || state.provider_idx == 3;

    // Name
    let name_line = Line::from(vec![
        Span::styled(" Name:      ", Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.name.is_empty() { "(Account name)" } else { &state.name },
            if state.active_field == 1 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(name_line), chunks[2]);
    app.hit_map.register(chunks[2], HitTarget::ModalAccountField(1));

    // URL
    let url_label = if is_oauth { " Endpoint:  " } else { " URL:       " };
    let url_line = Line::from(vec![
        Span::styled(url_label, Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            &state.url,
            if state.active_field == 2 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(url_line), chunks[3]);
    app.hit_map.register(chunks[3], HitTarget::ModalAccountField(2));

    // Username / Client ID
    let user_label = if is_oauth { " Client ID: " } else { " User / ID: " };
    let user_placeholder = if is_oauth {
        "(OAuth Client ID or Account Email)"
    } else {
        "(e.g. ID or username@server.com)"
    };
    let user_line = Line::from(vec![
        Span::styled(user_label, Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.username.is_empty() { user_placeholder } else { &state.username },
            if state.active_field == 3 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(user_line), chunks[4]);
    app.hit_map.register(chunks[4], HitTarget::ModalAccountField(3));

    // Password / Token
    let pw_label = if is_oauth { " Token/Key: " } else { " Password:  " };
    let pw_placeholder = if is_oauth {
        "(Access Token or API Secret)"
    } else {
        "(App-specific password)"
    };
    let masked_pw = "•".repeat(state.password.len());
    let pw_line = Line::from(vec![
        Span::styled(pw_label, Style::default().fg(Theme::TEXT_MUTED)),
        Span::styled(
            if state.password.is_empty() { pw_placeholder } else { &masked_pw },
            if state.active_field == 4 { active_style } else { normal_style },
        ),
    ]);
    f.render_widget(Paragraph::new(pw_line), chunks[5]);
    app.hit_map.register(chunks[5], HitTarget::ModalAccountField(4));

    // Help note
    let tip_text = match state.provider_idx {
        2 => " Google: Enter OAuth Bearer Token or API key for direct Google Calendar sync.",
        3 => " MS 365: Enter Microsoft Graph Access Token for Outlook calendar sync.",
        4 => " Samsung: Syncs via Samsung Cloud CalDAV or Samsung-Outlook Exchange integration.",
        0 => " Naver: Use your Naver ID and 2-step App Password.",
        1 => " Apple: Use your Apple ID and App-Specific Password.",
        _ => " CalDAV: Enter your custom CalDAV URL and server credentials.",
    };
    let note = vec![
        Line::from(Span::styled(tip_text, Style::default().fg(Theme::ACCENT).add_modifier(Modifier::ITALIC))),
    ];
    f.render_widget(Paragraph::new(note), chunks[6]);

    // Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(18), // [ Connect & Sync ]
            Constraint::Length(3),
            Constraint::Length(12), // [ Cancel ]
            Constraint::Min(1),
        ])
        .split(chunks[7]);

    let btn_conn = Paragraph::new(Line::from(vec![Span::raw(" [ Connect & Sync ] ")]))
        .style(Style::default().fg(Color::Black).bg(Theme::SECONDARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_conn, btn_chunks[1]);
    app.hit_map.register(btn_chunks[1], HitTarget::ModalAccountConnect);

    let btn_cancel = Paragraph::new(Line::from(vec![Span::raw(" [ Cancel ] ")]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_cancel, btn_chunks[3]);
    app.hit_map.register(btn_chunks[3], HitTarget::ModalCancel);
}

fn render_account_list(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let accounts = app.db.get_accounts().unwrap_or_default();
    let lang = app.lang;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Length(1), // Spacer
            Constraint::Min(6),    // List of accounts
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Bottom Buttons
        ])
        .split(area);

    let title_text = match lang {
        crate::i18n::Language::Ko => " 📋 연동된 캘린더 계정 목록 (삭제하려면 [ 삭제 ] 버튼 클릭):",
        crate::i18n::Language::En => " 📋 Configured Sync Accounts (Click [ Delete ] to remove):",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            title_text,
            Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD),
        )),
        chunks[0],
    );

    if accounts.is_empty() {
        let empty_msg = match lang {
            crate::i18n::Language::Ko => "  연동된 클라우드 계정이 없습니다. 아래의 '[ + 계정 추가 ]'를 클릭하여 Google, Microsoft, Samsung, Naver, Apple 캘린더를 연동하세요!",
            crate::i18n::Language::En => "  No cloud accounts linked yet. Click '[ + Add Account ]' below to link Google, Microsoft, Samsung, Naver, or Apple calendars!",
        };
        let p = Paragraph::new(Span::styled(
            empty_msg,
            Style::default().fg(Theme::TEXT_MUTED).add_modifier(Modifier::ITALIC),
        ));
        f.render_widget(p, chunks[2]);
    } else {
        for (idx, acc) in accounts.iter().enumerate() {
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
                    Constraint::Min(30),    // Left: Account details
                    Constraint::Length(12), // Right: [ 삭제 ] button
                ])
                .split(row_rect);

            let last_sync = acc.last_synced_at
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| match lang {
                    crate::i18n::Language::Ko => "동기화 이력 없음".to_string(),
                    crate::i18n::Language::En => "Never".to_string(),
                });

            let provider_color = match acc.provider {
                crate::model::ProviderType::Google => Color::Rgb(66, 133, 244),
                crate::model::ProviderType::Microsoft => Color::Rgb(0, 120, 212),
                crate::model::ProviderType::Samsung => Color::Rgb(3, 78, 162),
                crate::model::ProviderType::Naver => Color::Rgb(3, 199, 90),
                crate::model::ProviderType::Apple => Color::Rgb(90, 200, 250),
                crate::model::ProviderType::Local => Color::Cyan,
            };

            let is_selected = idx == state.selected_account_idx;
            let bullet = if is_selected { " ▶ " } else { " • " };

            let info_line = Line::from(vec![
                Span::styled(bullet, Style::default().fg(if is_selected { Theme::ACCENT } else { Theme::PRIMARY })),
                Span::styled(&acc.name, Style::default().fg(Theme::TEXT_HIGHLIGHT).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}]", acc.provider.as_str()), Style::default().fg(provider_color)),
                Span::styled(format!(" (ID: {})", acc.username), Style::default().fg(Theme::TEXT_NORMAL)),
                Span::styled(format!(" | {}", last_sync), Style::default().fg(Theme::TEXT_MUTED)),
            ]);
            f.render_widget(Paragraph::new(info_line), row_parts[0]);

            let del_label = match lang {
                crate::i18n::Language::Ko => "[ 삭제 ]",
                crate::i18n::Language::En => "[ Delete ]",
            };
            let del_btn = Paragraph::new(Line::from(vec![Span::raw(del_label)]))
                .style(
                    Style::default()
                        .fg(Theme::DANGER)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            f.render_widget(del_btn, row_parts[1]);
            app.hit_map.register(row_parts[1], HitTarget::ModalAccountDelete(acc.id.clone()));
        }
    }

    // Bottom Action Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // [ + 계정 추가 ]
            Constraint::Length(2),
            Constraint::Length(18), // [ ⟳ 동기화 실행 ]
            Constraint::Min(1),
            Constraint::Length(14), // [ 닫기 (q) ]
        ])
        .split(chunks[4]);

    let add_label = match lang {
        crate::i18n::Language::Ko => " [ + 계정 추가 ] ",
        crate::i18n::Language::En => " [ + Add Account ] ",
    };
    let btn_add = Paragraph::new(Line::from(vec![Span::raw(add_label)]))
        .style(Style::default().fg(Color::Black).bg(Theme::PRIMARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_add, btn_chunks[0]);
    app.hit_map.register(btn_chunks[0], HitTarget::ModalAccountAdd);

    let sync_label = match lang {
        crate::i18n::Language::Ko => " [ ⟳ 동기화 실행 ] ",
        crate::i18n::Language::En => " [ ⟳ Sync Now ] ",
    };
    let btn_sync = Paragraph::new(Line::from(vec![Span::raw(sync_label)]))
        .style(Style::default().fg(Color::Black).bg(Theme::SECONDARY).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(btn_sync, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::BtnSync);

    let close_label = match lang {
        crate::i18n::Language::Ko => " [ 닫기 (q) ] ",
        crate::i18n::Language::En => " [ Close (q) ] ",
    };
    let btn_close = Paragraph::new(Line::from(vec![Span::raw(close_label)]))
        .style(Theme::button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[4]);
    app.hit_map.register(btn_chunks[4], HitTarget::ModalClose);
}
