use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{AccountModalState, App};
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let popup_width = 84.min(area.width.saturating_sub(2));
    let popup_height = 24.min(area.height.saturating_sub(2));

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
    let title = if state.is_adding {
        lang.t().modal_account_add_title
    } else {
        lang.t().modal_account_manage_title
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

    if state.is_adding {
        render_add_form(f, app, state, inner);
    } else {
        render_account_list(f, app, state, inner);
    }
}

fn render_add_form(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    if state.is_oauth_waiting {
        render_oauth_waiting(f, app, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Preset buttons row
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Name
            Constraint::Length(1), // URL / Endpoint
            Constraint::Length(1), // Username / ID
            Constraint::Length(1), // Password / OAuth Token
            Constraint::Min(4),    // Help notes / Error alert box
            Constraint::Length(1), // Action Buttons
        ])
        .split(area);

    let theme = &app.theme;

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
        let is_row_focused = state.active_field == 0;
        let style = if is_selected && is_row_focused {
            Style::default()
                .fg(Color::Black)
                .bg(theme.text_highlight)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            theme.header_tab_active()
        } else {
            theme.header_tab_inactive()
        };
        let text = if is_selected && is_row_focused {
            format!("▶{}◀", label.trim())
        } else {
            label.to_string()
        };
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map
            .register(rect, HitTarget::ModalAccountPreset(idx));
    }

    let active_style = theme.active_input_style();
    let normal_style = Style::default().fg(theme.text_normal);

    let is_oauth = state.provider_idx == 2 || state.provider_idx == 3;

    // Name
    let name_line = Line::from(vec![
        Span::styled(" Name:      ", Style::default().fg(theme.text_muted)),
        Span::styled(
            if state.name.is_empty() {
                "(Account name)"
            } else {
                &state.name
            },
            if state.active_field == 1 {
                active_style
            } else {
                normal_style
            },
        ),
    ]);
    f.render_widget(Paragraph::new(name_line), chunks[2]);
    app.hit_map
        .register(chunks[2], HitTarget::ModalAccountField(1));

    // URL / Endpoint
    let url_label = if is_oauth {
        " Endpoint:  "
    } else {
        " CalDAV URL:"
    };
    let url_line = Line::from(vec![
        Span::styled(url_label, Style::default().fg(theme.text_muted)),
        Span::styled(
            if state.url.is_empty() {
                "(https://...)"
            } else {
                &state.url
            },
            if state.active_field == 2 {
                active_style
            } else {
                normal_style
            },
        ),
    ]);
    f.render_widget(Paragraph::new(url_line), chunks[3]);
    app.hit_map
        .register(chunks[3], HitTarget::ModalAccountField(2));

    // Username / Client ID
    let user_label = if is_oauth {
        " Client ID: "
    } else {
        " Username:  "
    };
    let user_placeholder = if state.provider_idx == 2 {
        "(GCP Desktop Client ID)"
    } else if is_oauth {
        "(Client ID / Key)"
    } else {
        "(User ID / Email)"
    };
    let user_line = Line::from(vec![
        Span::styled(user_label, Style::default().fg(theme.text_muted)),
        Span::styled(
            if state.username.is_empty() {
                user_placeholder
            } else {
                &state.username
            },
            if state.active_field == 3 {
                active_style
            } else {
                normal_style
            },
        ),
    ]);
    f.render_widget(Paragraph::new(user_line), chunks[4]);
    app.hit_map
        .register(chunks[4], HitTarget::ModalAccountField(3));

    // Password / Token
    let pw_label = if is_oauth {
        " Token/Sec: "
    } else {
        " Password:  "
    };
    let pw_display = if state.provider_idx == 2 && state.password.is_empty() {
        "(Client Secret / Optional)".to_string()
    } else if is_oauth && state.password.is_empty() {
        "(OAuth Token / Secret)".to_string()
    } else if state.password.is_empty() {
        "(App-specific password)".to_string()
    } else {
        "•".repeat(state.password.len().min(24))
    };
    let pw_line = Line::from(vec![
        Span::styled(pw_label, Style::default().fg(theme.text_muted)),
        Span::styled(
            pw_display,
            if state.active_field == 4 {
                active_style
            } else {
                normal_style
            },
        ),
    ]);
    f.render_widget(Paragraph::new(pw_line), chunks[5]);
    app.hit_map
        .register(chunks[5], HitTarget::ModalAccountField(4));

    // Provider guide tip or dedicated error box
    if state.is_oauth_waiting {
        let msg = match app.lang {
            crate::i18n::Language::Ko => {
                " ⏳ 브라우저에서 인증을 완료해주세요 (127.0.0.1:8088 대기 중)..."
            }
            crate::i18n::Language::Ja => {
                " ⏳ ブラウザで認証を完了してください (127.0.0.1:8088 待機中)..."
            }
            _ => " ⏳ Waiting for browser authentication on http://127.0.0.1:8088...",
        };
        let tip_widget = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                msg,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .wrap(Wrap { trim: true });
        f.render_widget(tip_widget, chunks[6]);
    } else if let Some(ref err) = state.error_msg {
        let err_title = match app.lang {
            crate::i18n::Language::Ko => " ⚠ 인증 오류 안내 (OAuth Error) ",
            crate::i18n::Language::Ja => " ⚠ 認証エラー (OAuth Error) ",
            _ => " ⚠ Authentication Error ",
        };

        let mut lines = Vec::new();
        for l in err.lines() {
            lines.push(Line::from(vec![Span::styled(
                format!(" {}", l),
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            )]));
        }

        let err_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightRed))
            .title(Span::styled(
                err_title,
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            ));

        let err_widget = Paragraph::new(lines)
            .block(err_block)
            .wrap(Wrap { trim: true });
        f.render_widget(err_widget, chunks[6]);
    } else {
        let msg = match state.provider_idx {
            0 => " Naver: ID and 2-step verification App Password (애플리케이션 비밀번호).",
            1 => " Apple iCloud: Apple ID email and iCloud App-Specific Password.",
            2 => match app.lang {
                crate::i18n::Language::Ko => " Google: GCP '데스크톱 앱' Client ID 입력 후 [ 🌐 1-Click Login ] 클릭 (RCAL_GOOGLE_CLIENT_ID)",
                crate::i18n::Language::Ja => " Google: GCPデスクトップClient ID入力後 [ 🌐 1-Click Login ]をクリック",
                _ => " Google: Enter GCP Desktop Client ID then click [ 🌐 1-Click Login ] (or env var)",
            },
            3 => match app.lang {
                crate::i18n::Language::Ko => " MS 365: [ 🌐 1-Click Login ] 클릭으로 브라우저 즉시 로그인 (공용 Client ID 지원)",
                crate::i18n::Language::Ja => " MS 365: [ 🌐 1-Click Login ]をクリックしてブラウザログイン",
                _ => " MS 365: Click [ 🌐 1-Click Login ] for browser sign-in (or enter manual token).",
            },
            4 => " Samsung: Connect via Samsung Cloud CalDAV endpoint or Microsoft Graph exchange.",
            _ => " CalDAV: Enter your own CalDAV server endpoint (Nextcloud, Baïkal, Radicale, etc).",
        };
        let tip_widget = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                msg,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::ITALIC),
            )),
        ])
        .wrap(Wrap { trim: true });
        f.render_widget(tip_widget, chunks[6]);
    }

    let is_oauth = state.provider_idx == 2 || state.provider_idx == 3;

    if is_oauth {
        // 3 Buttons for OAuth (Google & MS 365): [ 🌐 1-Click Login ] [ Manual Save ] [ Cancel ]
        let btn_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(22), // [ 🌐 1-Click Login ]
                Constraint::Length(2),
                Constraint::Length(18), // [ Manual Save ]
                Constraint::Length(2),
                Constraint::Length(12), // [ Cancel ]
                Constraint::Min(1),
            ])
            .split(chunks[7]);

        let is_oauth_focused = state.active_field == 5;
        let oauth_text = if is_oauth_focused {
            " ▶ 🌐 1-Click Login ◀ "
        } else {
            " 🌐 1-Click Login "
        };
        let btn_color = if state.provider_idx == 3 {
            Color::Rgb(0, 120, 212) // Microsoft Blue
        } else {
            Color::Rgb(66, 133, 244) // Google Blue
        };
        let oauth_style = if is_oauth_focused {
            Style::default()
                .fg(Color::Black)
                .bg(theme.text_highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
                .bg(btn_color)
                .add_modifier(Modifier::BOLD)
        };
        let btn_oauth = Paragraph::new(Line::from(vec![Span::raw(oauth_text)]))
            .style(oauth_style)
            .alignment(Alignment::Center);
        f.render_widget(btn_oauth, btn_chunks[1]);
        app.hit_map
            .register(btn_chunks[1], HitTarget::ModalAccountOAuthLogin);

        let is_manual_focused = state.active_field == 6;
        let manual_text = if is_manual_focused {
            " ▶ Manual Save ◀ "
        } else {
            " Manual Save "
        };
        let manual_style = if is_manual_focused {
            Style::default()
                .fg(Color::Black)
                .bg(theme.text_highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };
        let btn_manual = Paragraph::new(Line::from(vec![Span::raw(manual_text)]))
            .style(manual_style)
            .alignment(Alignment::Center);
        f.render_widget(btn_manual, btn_chunks[3]);
        app.hit_map
            .register(btn_chunks[3], HitTarget::ModalAccountConnect);

        let is_cancel_focused = state.active_field == 7;
        let cancel_text = if is_cancel_focused {
            " ▶ Cancel ◀ "
        } else {
            " Cancel "
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
        f.render_widget(btn_cancel, btn_chunks[5]);
        app.hit_map.register(btn_chunks[5], HitTarget::ModalCancel);
    } else {
        // Standard 2 Buttons: [ Connect & Sync ] [ Cancel ]
        let btn_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(22), // [ Connect & Sync ]
                Constraint::Length(3),
                Constraint::Length(14), // [ Cancel ]
                Constraint::Min(1),
            ])
            .split(chunks[7]);

        let is_conn_focused = state.active_field == 5;
        let conn_text = if is_conn_focused {
            " ▶ [ Connect & Sync ] ◀ "
        } else {
            " [ Connect & Sync ] "
        };
        let conn_style = if is_conn_focused {
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
        let btn_conn = Paragraph::new(Line::from(vec![Span::raw(conn_text)]))
            .style(conn_style)
            .alignment(Alignment::Center);
        f.render_widget(btn_conn, btn_chunks[1]);
        app.hit_map
            .register(btn_chunks[1], HitTarget::ModalAccountConnect);

        let is_cancel_focused = state.active_field == 6;
        let cancel_text = if is_cancel_focused {
            " ▶ [ Cancel ] ◀ "
        } else {
            " [ Cancel ] "
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
    }

    // Terminal cursor positioning on active input field
    let cursor_pos = match state.active_field {
        0 => {
            let active_rect = presets[state.provider_idx % 6].2;
            (active_rect.x + active_rect.width / 2, active_rect.y)
        }
        1 => {
            let val_w = Span::raw(&state.name).width() as u16;
            (chunks[2].x + 12 + val_w, chunks[2].y)
        }
        2 => {
            let val_w = Span::raw(&state.url).width() as u16;
            (chunks[3].x + 12 + val_w, chunks[3].y)
        }
        3 => {
            let val_w = Span::raw(&state.username).width() as u16;
            (chunks[4].x + 12 + val_w, chunks[4].y)
        }
        4 => {
            let val_w = state.password.len().min(24) as u16;
            (chunks[5].x + 12 + val_w, chunks[5].y)
        }
        5 => (chunks[7].x + chunks[7].width / 4, chunks[7].y),
        6 => (chunks[7].x + chunks[7].width / 2, chunks[7].y),
        7 => (chunks[7].x + (chunks[7].width * 3) / 4, chunks[7].y),
        _ => (chunks[2].x, chunks[2].y),
    };
    let safe_x = cursor_pos.0.min(area.right().saturating_sub(2));
    let safe_y = cursor_pos.1.min(area.bottom().saturating_sub(1));
    f.set_cursor_position((safe_x, safe_y));
}

fn render_account_list(f: &mut Frame, app: &mut App, state: &AccountModalState, area: Rect) {
    let accounts = app.db.get_accounts().unwrap_or_default();
    let lang = app.lang;
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Length(1), // Spacer
            Constraint::Min(4),    // List of accounts
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Keyboard shortcuts hint line
            Constraint::Length(1), // Bottom Buttons
        ])
        .split(area);

    let title_text = lang.t().account_list_header;
    f.render_widget(
        Paragraph::new(Span::styled(
            title_text,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        )),
        chunks[0],
    );

    if accounts.is_empty() {
        let empty_msg = lang.t().account_empty_msg;
        let p = Paragraph::new(Span::styled(
            empty_msg,
            Style::default()
                .fg(theme.text_muted)
                .add_modifier(Modifier::ITALIC),
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

            let last_sync = acc
                .last_synced_at
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| lang.t().account_never_synced.to_string());

            let provider_color = match acc.provider {
                crate::model::ProviderType::Google => Color::Rgb(66, 133, 244),
                crate::model::ProviderType::Microsoft => Color::Rgb(0, 120, 212),
                crate::model::ProviderType::Samsung => Color::Rgb(3, 78, 162),
                crate::model::ProviderType::Naver => Color::Rgb(15, 157, 88),
                crate::model::ProviderType::Apple => Color::Rgb(80, 160, 255),
                _ => Color::Cyan,
            };

            let is_selected = idx == state.selected_account_idx;
            let bullet = if is_selected { " ▶ " } else { " • " };

            let info_line = Line::from(vec![
                Span::styled(
                    bullet,
                    Style::default().fg(if is_selected {
                        theme.accent
                    } else {
                        theme.primary
                    }),
                ),
                Span::styled(
                    &acc.name,
                    Style::default()
                        .fg(theme.text_highlight)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" [{}]", acc.provider.as_str()),
                    Style::default().fg(provider_color),
                ),
                Span::styled(
                    format!(" (ID: {})", acc.username),
                    Style::default().fg(theme.text_normal),
                ),
                Span::styled(
                    format!(" | {}", last_sync),
                    Style::default().fg(theme.text_muted),
                ),
            ]);
            let row_style = if is_selected {
                Style::default().bg(theme.selected_bg)
            } else {
                Style::default()
            };
            f.render_widget(Paragraph::new(info_line).style(row_style), row_parts[0]);

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
                .register(row_parts[1], HitTarget::ModalAccountDelete(acc.id.clone()));
        }
    }

    // Keyboard shortcuts hint bar
    let hint_spans = match lang {
        crate::i18n::Language::Ko => vec![
            Span::styled(
                "[a/+]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 계정 추가   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[d]",
                Style::default()
                    .fg(theme.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 삭제   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[r]",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 동기화   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[↑/↓]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 계정 선택   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[Esc/q]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 닫기", Style::default().fg(theme.text_normal)),
        ],
        crate::i18n::Language::Ja => vec![
            Span::styled(
                "[a/+]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" アカウント追加   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[d]",
                Style::default()
                    .fg(theme.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 削除   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[r]",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 同期   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[↑/↓]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" アカウント選択   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[Esc/q]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" 閉じる", Style::default().fg(theme.text_normal)),
        ],
        _ => vec![
            Span::styled(
                "[a/+]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Add Account   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[d]",
                Style::default()
                    .fg(theme.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Delete   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[r]",
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Sync   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[↑/↓]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Select   ", Style::default().fg(theme.text_normal)),
            Span::styled(
                "[Esc/q]",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Close", Style::default().fg(theme.text_normal)),
        ],
    };
    f.render_widget(
        Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center),
        chunks[4],
    );

    // Bottom Action Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28), // [ + 계정 추가 (a/+) ]
            Constraint::Length(2),
            Constraint::Length(18), // [ ⟳ 동기화 실행 ]
            Constraint::Min(1),
            Constraint::Length(14), // [ 닫기 (q) ]
        ])
        .split(chunks[5]);

    let add_label = lang.t().account_btn_add;
    let btn_add = Paragraph::new(Line::from(vec![Span::raw(add_label)]))
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_add, btn_chunks[0]);
    app.hit_map
        .register(btn_chunks[0], HitTarget::ModalAccountAdd);

    let sync_label = lang.t().account_btn_sync_now;
    let btn_sync = Paragraph::new(Line::from(vec![Span::raw(sync_label)]))
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(btn_sync, btn_chunks[2]);
    app.hit_map.register(btn_chunks[2], HitTarget::BtnSync);

    let close_label = format!(" {} ", lang.btn_close());
    let btn_close = Paragraph::new(Line::from(vec![Span::raw(close_label)]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, btn_chunks[4]);
    app.hit_map.register(btn_chunks[4], HitTarget::ModalClose);

    if accounts.is_empty() {
        f.set_cursor_position((btn_chunks[0].x + btn_chunks[0].width / 2, btn_chunks[0].y));
    } else {
        let sel_idx = state
            .selected_account_idx
            .min(accounts.len().saturating_sub(1));
        let cursor_y = chunks[2].y + sel_idx as u16;
        f.set_cursor_position((chunks[2].x + 1, cursor_y));
    }
}

fn render_oauth_waiting(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Browser status
            Constraint::Length(2), // Local listener URL
            Constraint::Min(4),    // Instructions
            Constraint::Length(1), // Cancel button
        ])
        .split(area);

    let title_line = Line::from(vec![Span::styled(
        " 🌐 Google Calendar 1-Click OAuth2 Authentication ",
        Style::default()
            .fg(Color::Rgb(66, 133, 244))
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(
        Paragraph::new(title_line).alignment(Alignment::Center),
        chunks[1],
    );

    let status_line = Line::from(vec![Span::styled(
        "⏳ Your default web browser has been opened...",
        Style::default()
            .fg(theme.text_highlight)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(
        Paragraph::new(status_line).alignment(Alignment::Center),
        chunks[3],
    );

    let url_line = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "Local callback listener: ",
            Style::default().fg(theme.text_muted),
        ),
        Span::styled(
            "http://127.0.0.1:8088/oauth/callback",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ),
    ])])
    .alignment(Alignment::Center);
    f.render_widget(url_line, chunks[4]);

    let info_lines = vec![
        Line::from(Span::styled(
            "1. Sign in with your Google account in the browser window.",
            Style::default().fg(theme.text_normal),
        )),
        Line::from(Span::styled(
            "2. Grant calendar read/write access permissions to rcal.",
            Style::default().fg(theme.text_normal),
        )),
        Line::from(Span::styled(
            "3. Once approved, rcal will sync automatically!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    f.render_widget(
        Paragraph::new(info_lines).alignment(Alignment::Center),
        chunks[5],
    );

    // Cancel Button
    let cancel_btn = Paragraph::new(Line::from(vec![Span::raw(" [ Cancel / Esc ] ")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(cancel_btn, chunks[6]);
    app.hit_map.register(chunks[6], HitTarget::ModalCancel);
}
