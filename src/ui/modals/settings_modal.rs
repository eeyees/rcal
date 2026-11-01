use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, SettingsSection};
use crate::model::WeekStartDay;
use crate::mouse::HitTarget;
use crate::ui::theme::{Theme, ThemeCategory, ThemeMode};

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 90.min(area.width.saturating_sub(2));
    let popup_height = 29.min(area.height.saturating_sub(1));

    let popup_rect = Rect {
        x: area.x + (area.width.saturating_sub(popup_width)) / 2,
        y: area.y + (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_rect);
    app.hit_map.register(popup_rect, HitTarget::ModalConsume);

    let lang = app.lang;
    let theme = app.theme;
    let t = lang.t();
    let title = t.modal_settings_title;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Subtitle / Dialog Header (0, y = inner.y)
            Constraint::Length(1), // Top Tabs Bar (1, y = inner.y + 1)
            Constraint::Length(1), // Divider (2, y = inner.y + 2)
            Constraint::Min(19),   // Active Tab Expanded Content Area (3, y = inner.y + 3)
            Constraint::Length(1), // Spacer (4)
            Constraint::Length(1), // Keyboard navigation guide (5)
            Constraint::Length(1), // Close button (6)
        ])
        .split(inner);

    // Header info line inside modal (consume click, no button hit target)
    let header_desc = match lang {
        crate::i18n::Language::Ko => {
            " ⚙  환경설정 메뉴 | 상단 탭을 선택하여 각 항목별 상세 설정을 구성하세요."
        }
        crate::i18n::Language::Ja => {
            " ⚙  環境設定メニュー | 上部のタブを選択して各項目の詳細設定を行ってください。"
        }
        _ => " ⚙  Settings Menu | Select tabs above to customize options and preferences.",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            header_desc,
            Style::default().fg(theme.text_dim),
        )),
        main_chunks[0],
    );

    // Active tab determines which content is shown.
    // When Close button is focused, continue displaying the previously active tab (Language by default).
    let active_tab = match app.settings_section {
        SettingsSection::Close => SettingsSection::Language,
        other => other,
    };

    // =========================================================================
    // 1. Top Tabs Bar
    // =========================================================================
    render_tabs_bar(f, app, main_chunks[1], active_tab);

    // 2. Divider line under tabs
    let divider_style = Style::default().fg(theme.border);
    let divider_char = "─".repeat(main_chunks[2].width as usize);
    f.render_widget(
        Paragraph::new(divider_char).style(divider_style),
        main_chunks[2],
    );

    // =========================================================================
    // 3. Tab Content Area
    // =========================================================================
    let mut week_cursor_rect = Rect::default();
    let mut notif_cursor_rect = Rect::default();
    let mut theme_cursor_rect = Rect::default();
    let mut lang_cursor_rect = Rect::default();

    match active_tab {
        SettingsSection::WeekStart => {
            week_cursor_rect = render_week_start_tab(f, app, main_chunks[3]);
        }
        SettingsSection::Notifications => {
            notif_cursor_rect = render_notifications_tab(f, app, main_chunks[3]);
        }
        SettingsSection::Theme => {
            theme_cursor_rect = render_theme_tab(f, app, main_chunks[3]);
        }
        SettingsSection::Language => {
            lang_cursor_rect = render_language_tab(f, app, main_chunks[3]);
        }
        SettingsSection::Close => {
            lang_cursor_rect = render_language_tab(f, app, main_chunks[3]);
        }
    }

    // =========================================================================
    // 4. Footer & Close Button
    // =========================================================================
    let hint_line = Line::from(vec![
        Span::styled(
            t.settings_hint_shortcuts,
            Style::default().fg(theme.text_muted),
        ),
        Span::styled(
            "[Tab/S-Tab]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(match lang {
            crate::i18n::Language::Ko => " 탭이동  ",
            crate::i18n::Language::Ja => " タブ移動  ",
            _ => " Tab Nav  ",
        }),
        Span::styled(
            "[1~4]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(match lang {
            crate::i18n::Language::Ko => " 바로가기  ",
            crate::i18n::Language::Ja => " ショートカット  ",
            _ => " Shortcut  ",
        }),
        Span::styled(
            "[←/→/↑/↓]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(match lang {
            crate::i18n::Language::Ko => " 항목변경  ",
            crate::i18n::Language::Ja => " 項目変更  ",
            _ => " Change  ",
        }),
        Span::styled(
            "[Esc/q]",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(match lang {
            crate::i18n::Language::Ko => " 닫기",
            crate::i18n::Language::Ja => " 閉じる",
            _ => " Close",
        }),
    ]);
    f.render_widget(
        Paragraph::new(hint_line).alignment(Alignment::Center),
        main_chunks[5],
    );

    // Close button
    let is_close_focused = app.settings_section == SettingsSection::Close;
    let close_style = if is_close_focused {
        Style::default()
            .fg(if theme.mode.is_light() {
                Color::White
            } else {
                Color::Black
            })
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        theme.button_style()
    };
    let close_text = if is_close_focused {
        format!("▶ {} ◀", t.settings_btn_close)
    } else {
        t.settings_btn_close.to_string()
    };
    let close_btn = Paragraph::new(Line::from(vec![Span::raw(close_text)]))
        .style(close_style)
        .alignment(Alignment::Center);
    f.render_widget(close_btn, main_chunks[6]);
    app.hit_map.register(main_chunks[6], HitTarget::ModalClose);

    // =========================================================================
    // 5. Cursor Placement per Section
    // =========================================================================
    match app.settings_section {
        SettingsSection::WeekStart => {
            f.set_cursor_position((
                week_cursor_rect.x + week_cursor_rect.width / 2,
                week_cursor_rect.y,
            ));
        }
        SettingsSection::Notifications => {
            f.set_cursor_position((
                notif_cursor_rect.x + notif_cursor_rect.width / 2,
                notif_cursor_rect.y,
            ));
        }
        SettingsSection::Theme => {
            f.set_cursor_position((
                theme_cursor_rect.x + theme_cursor_rect.width / 2,
                theme_cursor_rect.y,
            ));
        }
        SettingsSection::Language => {
            f.set_cursor_position((
                lang_cursor_rect.x + lang_cursor_rect.width / 2,
                lang_cursor_rect.y,
            ));
        }
        SettingsSection::Close => {
            let rect = main_chunks[6];
            f.set_cursor_position((rect.x + rect.width / 2, rect.y));
        }
    }
}

// =============================================================================
// Helper: Render Top Tabs Bar
// =============================================================================
fn render_tabs_bar(f: &mut Frame, app: &mut App, area: Rect, active_tab: SettingsSection) {
    let theme = app.theme;
    let lang = app.lang;

    let tabs = SettingsSection::TABS;
    let tab_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(area);

    for (idx, sec) in tabs.iter().enumerate() {
        let is_selected = active_tab == *sec;
        let prefix = if is_selected { "● " } else { "○ " };
        let title = sec.tab_title(lang);
        let text = format!("{}{}", prefix, title);

        let style = if is_selected {
            Style::default()
                .fg(if theme.mode.is_light() {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };

        let rect = tab_chunks[idx];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalSettingTab(*sec));
    }
}

// =============================================================================
// Tab 1: Week Start Day Content
// =============================================================================
fn render_week_start_tab(f: &mut Frame, app: &mut App, area: Rect) -> Rect {
    let theme = app.theme;
    let lang = app.lang;
    let t = lang.t();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title (y = inner.y + 3)
            Constraint::Length(1), // Subtitle (y = inner.y + 4)
            Constraint::Length(1), // Spacer (y = inner.y + 5)
            Constraint::Length(1), // Buttons (y = inner.y + 6)
            Constraint::Length(1), // Spacer (y = inner.y + 7)
            Constraint::Length(8), // Calendar Preview Box (y = inner.y + 8..15)
            Constraint::Length(1), // Spacer (y = inner.y + 16)
            Constraint::Length(4), // Tips & Guide Box (y = inner.y + 17..20)
            Constraint::Min(0),
        ])
        .split(area);

    // Title & Subtitle
    let is_focused = app.settings_section == SettingsSection::WeekStart;
    let title_style = if is_focused {
        Style::default()
            .fg(theme.text_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD)
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("▶ {}", t.settings_week_start_title),
            title_style,
        )),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Span::styled(
            t.settings_week_start_desc,
            Style::default().fg(theme.text_muted),
        )),
        chunks[1],
    );

    // 3 Buttons
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26), // Sunday
            Constraint::Length(2),
            Constraint::Length(26), // Monday
            Constraint::Length(2),
            Constraint::Length(26), // Saturday
            Constraint::Min(0),
        ])
        .split(chunks[3]);

    let options = match lang {
        crate::i18n::Language::Ko => [
            (WeekStartDay::Sunday, "일요일 (Sun)", "s"),
            (WeekStartDay::Monday, "월요일 (Mon)", "m"),
            (WeekStartDay::Saturday, "토요일 (Sat)", "t"),
        ],
        crate::i18n::Language::Ja => [
            (WeekStartDay::Sunday, "日曜日 (Sun)", "s"),
            (WeekStartDay::Monday, "月曜日 (Mon)", "m"),
            (WeekStartDay::Saturday, "土曜日 (Sat)", "t"),
        ],
        _ => [
            (WeekStartDay::Sunday, "Sunday (Sun)", "s"),
            (WeekStartDay::Monday, "Monday (Mon)", "m"),
            (WeekStartDay::Saturday, "Saturday (Sat)", "t"),
        ],
    };

    let mut active_rect = btn_chunks[0];
    for (idx, (day, label, key)) in options.iter().enumerate() {
        let is_selected = app.week_start_day == *day;
        let prefix = if is_selected { "● " } else { "○ " };
        let text = format!("[ {}{} [{}] ]", prefix, label, key);

        let style = if is_selected {
            Style::default()
                .fg(if theme.mode.is_light() {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            theme.button_style()
        };

        let rect = btn_chunks[idx * 2];
        if is_selected {
            active_rect = rect;
        }
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map
            .register(rect, HitTarget::ModalSettingWeekStart(*day));
    }

    // Calendar Preview Box
    let preview_title = match lang {
        crate::i18n::Language::Ko => " 📅 주간 배치 실시간 미리보기 (Live Week Preview) ",
        crate::i18n::Language::Ja => " 📅 週の配置プレビュー (Live Week Preview) ",
        _ => " 📅 Live Week Layout Preview ",
    };
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            preview_title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let p_inner = preview_block.inner(chunks[5]);
    f.render_widget(preview_block, chunks[5]);

    let (headers, dates) = match app.week_start_day {
        WeekStartDay::Sunday => (
            vec![
                Span::styled(
                    " [일 Sun] ",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("    월 Mon      화 Tue      수 Wed      목 Thu      금 Fri    "),
                Span::styled(
                    " [토 Sat] ",
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ],
            vec![
                Span::styled("   (주말)  ", Style::default().fg(Color::LightRed)),
                Span::raw("                                                          "),
                Span::styled("  (주말)  ", Style::default().fg(Color::LightCyan)),
            ],
        ),
        WeekStartDay::Monday => (
            vec![
                Span::raw("   월 Mon      화 Tue      수 Wed      목 Thu      금 Fri    "),
                Span::styled(
                    " [토 Sat] ",
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " [일 Sun] ",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
            ],
            vec![
                Span::raw("                                                          "),
                Span::styled("  (주말)  ", Style::default().fg(Color::LightCyan)),
                Span::styled("  (주말)  ", Style::default().fg(Color::LightRed)),
            ],
        ),
        WeekStartDay::Saturday => (
            vec![
                Span::styled(
                    " [토 Sat] ",
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " [일 Sun] ",
                    Style::default()
                        .fg(Color::LightRed)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("    월 Mon      화 Tue      수 Wed      목 Thu      금 Fri    "),
            ],
            vec![
                Span::styled("   (주말)  ", Style::default().fg(Color::LightCyan)),
                Span::styled("  (주말)  ", Style::default().fg(Color::LightRed)),
                Span::raw("                                                          "),
            ],
        ),
    };

    let p_lines = vec![
        Line::from(vec![Span::raw("")]),
        Line::from(headers),
        Line::from(dates),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "  Day 1        Day 2       Day 3       Day 4       Day 5       Day 6       Day 7  ",
            Style::default().fg(theme.text_dim),
        )]),
    ];
    f.render_widget(
        Paragraph::new(p_lines).alignment(Alignment::Center),
        p_inner,
    );

    // Tips Box
    let guide_title = match lang {
        crate::i18n::Language::Ko => " 💡 주 시작 요일 안내 ",
        crate::i18n::Language::Ja => " 💡 週の開始曜日ガイド ",
        _ => " 💡 Week Start Guide ",
    };
    let guide_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            guide_title,
            Style::default().fg(theme.text_highlight),
        ));
    let g_inner = guide_block.inner(chunks[7]);
    f.render_widget(guide_block, chunks[7]);

    let g_lines = match lang {
        crate::i18n::Language::Ko => vec![
            Line::from(vec![
                Span::styled("• 단축키: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw("[s] 일요일, [m] 월요일, [t] 토요일 또는 [←/→/↑/↓] 키로 즉시 변경 가능합니다."),
            ]),
            Line::from(vec![
                Span::styled("• 반영범위: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw("월간 뷰 그리드, 주간 뷰 타임라인, 미니 달력 등 모든 캘린더 화면에 즉시 적용됩니다."),
            ]),
        ],
        crate::i18n::Language::Ja => vec![
            Line::from(vec![
                Span::styled("• ショートカット: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw("[s] 日曜日、[m] 月曜日、[t] 土曜日、または [←/→/↑/↓] で即時切替可能。"),
            ]),
            Line::from(vec![
                Span::styled("• 反映範囲: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw("月間グリッド・週間タイムライン・ミニカレンダーの表示が即座に更新されます。"),
            ]),
        ],
        _ => vec![
            Line::from(vec![
                Span::styled("• Shortcuts: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw("[s] Sunday, [m] Monday, [t] Saturday, or [←/→/↑/↓] arrow keys."),
            ]),
            Line::from(vec![
                Span::styled("• Applies to: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw("Month view grid, Week timeline, and Mini calendars across the application."),
            ]),
        ],
    };
    f.render_widget(Paragraph::new(g_lines), g_inner);

    active_rect
}

// =============================================================================
// Tab 2: Desktop Notifications Content
// =============================================================================
fn render_notifications_tab(f: &mut Frame, app: &mut App, area: Rect) -> Rect {
    let theme = app.theme;
    let lang = app.lang;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title & Test button row (y = inner.y + 3)
            Constraint::Length(1), // Subtitle (y = inner.y + 4)
            Constraint::Length(1), // Spacer (y = inner.y + 5)
            Constraint::Length(1), // Section 1 label: 알림 및 소리 제어 (y = inner.y + 6)
            Constraint::Length(1), // Toggle & Sound buttons (y = inner.y + 7)
            Constraint::Length(1), // Spacer (y = inner.y + 8)
            Constraint::Length(1), // Lead times label (y = inner.y + 9)
            Constraint::Length(1), // Lead times: 5 buttons (y = inner.y + 10)
            Constraint::Length(1), // Spacer (y = inner.y + 11)
            Constraint::Length(6), // Summary Box (y = inner.y + 12..17)
            Constraint::Min(0),
        ])
        .split(area);

    // Title & Test button
    let is_focused = app.settings_section == SettingsSection::Notifications;
    let title_style = if is_focused {
        Style::default()
            .fg(theme.text_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD)
    };

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(26)])
        .split(chunks[0]);

    let notif_title = match lang {
        crate::i18n::Language::Ko => "▶ 🔔  바탕화면 알림 (Desktop Notifications)",
        crate::i18n::Language::Ja => "▶ 🔔  デスクトップ通知 (Desktop Notifications)",
        _ => "▶ 🔔  Desktop Notifications",
    };
    f.render_widget(
        Paragraph::new(Span::styled(notif_title, title_style)),
        header_chunks[0],
    );

    let test_btn_label = match lang {
        crate::i18n::Language::Ko => "[ 🔔  테스트 발송 (d) ]",
        crate::i18n::Language::Ja => "[ 🔔  テスト送信 (d) ]",
        _ => "[ 🔔  Test Alert (d) ]",
    };
    let btn_test = Paragraph::new(Line::from(vec![Span::raw(test_btn_label)]))
        .style(theme.button_style())
        .alignment(Alignment::Right);
    f.render_widget(btn_test, header_chunks[1]);
    app.hit_map
        .register(header_chunks[1], HitTarget::ModalSettingNotificationTest);

    // Subtitle
    let desc_str = match lang {
        crate::i18n::Language::Ko => {
            "일정 시작 전 시스템 팝업 알림과 효과음으로 일정을 놓치지 않도록 알려줍니다."
        }
        crate::i18n::Language::Ja => "予定開始前にシステム通知と効果音でリマインドします。",
        _ => "Receive desktop system alerts and sounds before scheduled events start.",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            desc_str,
            Style::default().fg(theme.text_muted),
        )),
        chunks[1],
    );

    // Section 1 Label
    let label_sec1 = match lang {
        crate::i18n::Language::Ko => "1. 알림 활성화 및 효과음 설정:",
        crate::i18n::Language::Ja => "1. 通知の有効化とサウンド設定:",
        _ => "1. Notification & Sound Controls:",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            label_sec1,
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )),
        chunks[3],
    );

    // Toggle & Sound Controls (chunks[4])
    let ctrl_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Toggle button
            Constraint::Length(2),
            Constraint::Length(22), // Sound button
            Constraint::Min(0),
        ])
        .split(chunks[4]);

    // 1. Toggle Button
    let notif_enabled = app.notification_config.enabled;
    let toggle_label = match lang {
        crate::i18n::Language::Ko => {
            if notif_enabled {
                "● 알림 켬 [n]"
            } else {
                "○ 알림 끔 [n]"
            }
        }
        crate::i18n::Language::Ja => {
            if notif_enabled {
                "● 通知オン [n]"
            } else {
                "○ 通知オフ [n]"
            }
        }
        _ => {
            if notif_enabled {
                "● Alerts ON [n]"
            } else {
                "○ Alerts OFF [n]"
            }
        }
    };
    let toggle_style = if notif_enabled {
        Style::default()
            .fg(if theme.mode.is_light() {
                Color::White
            } else {
                Color::Black
            })
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        theme.button_style()
    };
    let toggle_p = Paragraph::new(Line::from(vec![Span::raw(toggle_label)]))
        .style(toggle_style)
        .alignment(Alignment::Center);
    f.render_widget(toggle_p, ctrl_chunks[0]);
    app.hit_map
        .register(ctrl_chunks[0], HitTarget::ModalSettingNotificationToggle);

    // 2. Sound Button
    let sound_enabled = app.notification_config.sound;
    let sound_label = match lang {
        crate::i18n::Language::Ko => {
            if sound_enabled {
                "● 🔊  소리 켬 [o]"
            } else {
                "○ 🔇  무음 [o]"
            }
        }
        crate::i18n::Language::Ja => {
            if sound_enabled {
                "● 🔊  音オン [o]"
            } else {
                "○ 🔇  消音 [o]"
            }
        }
        _ => {
            if sound_enabled {
                "● 🔊  Sound ON [o]"
            } else {
                "○ 🔇  Mute [o]"
            }
        }
    };
    let sound_style = if sound_enabled {
        Style::default()
            .fg(theme.text_highlight)
            .bg(theme.button_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        theme.button_style()
    };
    let sound_p = Paragraph::new(Line::from(vec![Span::raw(sound_label)]))
        .style(sound_style)
        .alignment(Alignment::Center);
    f.render_widget(sound_p, ctrl_chunks[2]);
    app.hit_map
        .register(ctrl_chunks[2], HitTarget::ModalSettingNotificationSound);

    // Section 2 Label: Lead Times
    let label_sec2 = match lang {
        crate::i18n::Language::Ko => "2. 사전 알림 발송 타이밍 (←/→ 키로 변경 가능):",
        crate::i18n::Language::Ja => "2. 事前通知タイミング (←/→ キーで変更):",
        _ => "2. Advance Alert Lead Time (navigate with ←/→):",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            label_sec2,
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )),
        chunks[6],
    );

    // Lead times buttons (chunks[7])
    let lead_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14), // 0m
            Constraint::Length(1),
            Constraint::Length(14), // 5m
            Constraint::Length(1),
            Constraint::Length(14), // 10m
            Constraint::Length(1),
            Constraint::Length(14), // 15m
            Constraint::Length(1),
            Constraint::Length(14), // 30m
            Constraint::Min(0),
        ])
        .split(chunks[7]);

    let lead_opts = [
        (
            0u32,
            match lang {
                crate::i18n::Language::Ko => "정시 (0m)",
                crate::i18n::Language::Ja => "定時 (0m)",
                _ => "0m (At start)",
            },
        ),
        (
            5u32,
            match lang {
                crate::i18n::Language::Ko => "5분 전",
                crate::i18n::Language::Ja => "5分前",
                _ => "5m before",
            },
        ),
        (
            10u32,
            match lang {
                crate::i18n::Language::Ko => "10분 전",
                crate::i18n::Language::Ja => "10分前",
                _ => "10m before",
            },
        ),
        (
            15u32,
            match lang {
                crate::i18n::Language::Ko => "15분 전",
                crate::i18n::Language::Ja => "15分前",
                _ => "15m before",
            },
        ),
        (
            30u32,
            match lang {
                crate::i18n::Language::Ko => "30분 전",
                crate::i18n::Language::Ja => "30分前",
                _ => "30m before",
            },
        ),
    ];

    for (idx, (mins, label)) in lead_opts.iter().enumerate() {
        let is_selected = app.notification_config.lead_minutes == *mins;
        let prefix = if is_selected { "● " } else { "○ " };
        let text = format!("{}{}", prefix, label);
        let style = if is_selected {
            Style::default()
                .fg(if theme.mode.is_light() {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };
        let rect = lead_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map
            .register(rect, HitTarget::ModalSettingNotificationLead(*mins));
    }

    // Status Summary Box (chunks[9])
    let box_title = match lang {
        crate::i18n::Language::Ko => " 🔔 알림 동작 상태 요약 (Live Status) ",
        crate::i18n::Language::Ja => " 🔔 通知ステータスサマリー (Live Status) ",
        _ => " 🔔 Notification Status Summary ",
    };
    let sum_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            box_title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let s_inner = sum_block.inner(chunks[9]);
    f.render_widget(sum_block, chunks[9]);

    let status_line1 = if notif_enabled {
        Span::styled(
            "  ● 알림 상태: 활성화됨 (시스템 트레이 알림 동작 중)",
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            "  ○ 알림 상태: 꺼짐 (알림이 발생하지 않습니다)",
            Style::default().fg(Color::LightRed),
        )
    };

    let status_line2 = format!(
        "  ⏱️ 발송 타이밍: 이벤트 시작 {}분 전에 팝업 발생",
        app.notification_config.lead_minutes
    );
    let status_line3 = if sound_enabled {
        "  🔊 사운드: 알림음 재생 켜짐 (효과음 포함)"
    } else {
        "  🔇 사운드: 무음 (팝업 배너만 표시)"
    };
    let status_line4 =
        "  💡 단축키: [n] 알림 토글   [o] 소리 토글   [←/→] 사전시간 변경   [d] 테스트 발송";

    let s_lines = vec![
        Line::from(vec![status_line1]),
        Line::from(vec![Span::styled(
            status_line2,
            Style::default().fg(theme.text_normal),
        )]),
        Line::from(vec![Span::styled(
            status_line3,
            Style::default().fg(theme.text_normal),
        )]),
        Line::from(vec![Span::styled(
            status_line4,
            Style::default().fg(theme.text_dim),
        )]),
    ];
    f.render_widget(Paragraph::new(s_lines), s_inner);

    ctrl_chunks[0]
}

// =============================================================================
// Tab 3: Color Themes Content
// =============================================================================
fn render_theme_tab(f: &mut Frame, app: &mut App, area: Rect) -> Rect {
    let theme = app.theme;
    let lang = app.lang;
    let t = lang.t();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title & Theme Modal button (y = inner.y + 3)
            Constraint::Length(1), // Current theme badge & info (y = inner.y + 4)
            Constraint::Length(1), // Spacer (y = inner.y + 5)
            Constraint::Length(1), // Category Tabs [Prev] [Dark] [Nature] [Vibrant] [Light] [Next] (y = inner.y + 6)
            Constraint::Length(1), // Spacer (y = inner.y + 7)
            Constraint::Length(1), // Theme Grid Row 1 (y = inner.y + 8)
            Constraint::Length(1), // Theme Grid Row 2 (y = inner.y + 9)
            Constraint::Length(1), // Spacer (y = inner.y + 10)
            Constraint::Length(5), // Color Palette Preview Box (y = inner.y + 11..15)
            Constraint::Length(1), // Spacer (y = inner.y + 16)
            Constraint::Length(1), // Config file path (y = inner.y + 17)
            Constraint::Min(0),
        ])
        .split(area);

    // Title & Modal Button
    let is_focused = app.settings_section == SettingsSection::Theme;
    let title_style = if is_focused {
        Style::default()
            .fg(theme.text_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD)
    };

    let all_themes = ThemeMode::all();
    let current_idx = all_themes
        .iter()
        .position(|m| *m == app.theme.mode)
        .unwrap_or(0);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(26)])
        .split(chunks[0]);

    f.render_widget(
        Paragraph::new(Span::styled(
            format!("▶ {}", t.settings_theme_title),
            title_style,
        )),
        header_chunks[0],
    );

    let open_modal_label = match lang {
        crate::i18n::Language::Ko => "[ 🎨  전체 테마 모달 (T) ]",
        crate::i18n::Language::Ja => "[ 🎨  テーマ一覧 (T) ]",
        _ => "[ 🎨  Theme Modal (T) ]",
    };
    let btn_open_modal = Paragraph::new(Line::from(vec![Span::raw(open_modal_label)]))
        .style(theme.button_style())
        .alignment(Alignment::Right);
    f.render_widget(btn_open_modal, header_chunks[1]);
    app.hit_map.register(header_chunks[1], HitTarget::BtnTheme);

    // Current theme badge
    let current_theme_badge = format!(
        "  [ {}/{} ] {}",
        current_idx + 1,
        all_themes.len(),
        app.theme.mode.label(lang)
    );
    let theme_desc_line = Line::from(vec![
        Span::styled(t.settings_theme_desc, Style::default().fg(theme.text_muted)),
        Span::styled(
            current_theme_badge,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(theme_desc_line), chunks[1]);

    // Category Tabs: [◀ Prev] | [1:Dark] [2:Nature] [3:Vibrant] [4:Light] | [Next ▶]
    let cat_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // [◀ Prev]
            Constraint::Length(1),
            Constraint::Length(13), // Dark
            Constraint::Length(1),
            Constraint::Length(13), // Nature
            Constraint::Length(1),
            Constraint::Length(13), // Vibrant
            Constraint::Length(1),
            Constraint::Length(13), // Light
            Constraint::Length(1),
            Constraint::Length(10), // [Next ▶]
            Constraint::Min(0),
        ])
        .split(chunks[3]);

    let prev_btn = Paragraph::new(Line::from(vec![Span::raw("◀ 이전 (c)")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(prev_btn, cat_chunks[0]);
    app.hit_map
        .register(cat_chunks[0], HitTarget::ModalSettingThemePrev);

    let categories = [
        (ThemeCategory::Dark, "다크 (10)"),
        (ThemeCategory::Nature, "내추럴 (9)"),
        (ThemeCategory::Vibrant, "비비드 (8)"),
        (ThemeCategory::Light, "라이트 (10)"),
    ];

    for (idx, (cat, name)) in categories.iter().enumerate() {
        let is_active_cat = app.theme_category == *cat;
        let prefix = if is_active_cat { "● " } else { "○ " };
        let text = format!("{}{}", prefix, name);
        let style = if is_active_cat {
            Style::default()
                .fg(if theme.mode.is_light() {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };

        let rect = cat_chunks[2 + idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map
            .register(rect, HitTarget::ModalSettingThemeCategory(*cat));
    }

    let next_btn = Paragraph::new(Line::from(vec![Span::raw("다음 (C) ▶")]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(next_btn, cat_chunks[10]);
    app.hit_map
        .register(cat_chunks[10], HitTarget::ModalSettingThemeNext);

    // Active Category Themes Grid (Row 1 & Row 2)
    let cat_themes = app.theme_category.themes();
    let num_in_cat = cat_themes.len();
    let row1_count = (num_in_cat + 1) / 2;
    let row2_count = num_in_cat.saturating_sub(row1_count);

    let mut selected_theme_rect: Option<Rect> = None;

    // Row 1 themes (y = inner.y + 8)
    let row1_themes = &cat_themes[0..row1_count.min(num_in_cat)];
    let mut row1_constraints = Vec::with_capacity(row1_themes.len() * 2 + 1);
    for _ in 0..row1_themes.len() {
        row1_constraints.push(Constraint::Length(16));
        row1_constraints.push(Constraint::Length(1));
    }
    row1_constraints.push(Constraint::Min(0));

    let row1_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(row1_constraints)
        .split(chunks[5]);

    for (idx, mode) in row1_themes.iter().enumerate() {
        let is_selected = app.theme.mode == *mode;
        let prefix = if is_selected { "● " } else { "○ " };
        let label = format!("{}{}", prefix, mode.short_name(lang));
        let sample = Theme::from_mode(*mode);

        let style = if is_selected {
            Style::default()
                .fg(if mode.is_light() {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(sample.primary).bg(theme.button_bg)
        };

        let rect = row1_chunks[idx * 2];
        if is_selected {
            selected_theme_rect = Some(rect);
        }
        let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map
            .register(rect, HitTarget::ModalSettingTheme(*mode));
    }

    // Row 2 themes (y = inner.y + 9)
    if row2_count > 0 {
        let row2_themes = &cat_themes[row1_count..num_in_cat];
        let mut row2_constraints = Vec::with_capacity(row2_themes.len() * 2 + 1);
        for _ in 0..row2_themes.len() {
            row2_constraints.push(Constraint::Length(16));
            row2_constraints.push(Constraint::Length(1));
        }
        row2_constraints.push(Constraint::Min(0));

        let row2_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(row2_constraints)
            .split(chunks[6]);

        for (idx, mode) in row2_themes.iter().enumerate() {
            let is_selected = app.theme.mode == *mode;
            let prefix = if is_selected { "● " } else { "○ " };
            let label = format!("{}{}", prefix, mode.short_name(lang));
            let sample = Theme::from_mode(*mode);

            let style = if is_selected {
                Style::default()
                    .fg(if mode.is_light() {
                        Color::White
                    } else {
                        Color::Black
                    })
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(sample.primary).bg(theme.button_bg)
            };

            let rect = row2_chunks[idx * 2];
            if is_selected {
                selected_theme_rect = Some(rect);
            }
            let p = Paragraph::new(Line::from(vec![Span::raw(label)]))
                .style(style)
                .alignment(Alignment::Center);
            f.render_widget(p, rect);
            app.hit_map
                .register(rect, HitTarget::ModalSettingTheme(*mode));
        }
    }

    // Palette Preview Box (chunks[8])
    let preview_title = match lang {
        crate::i18n::Language::Ko => " 🎨 현재 테마 팔레트 미리보기 (Live Palette Preview) ",
        crate::i18n::Language::Ja => " 🎨 現在のテーマパレット (Live Palette Preview) ",
        _ => " 🎨 Live Theme Palette Preview ",
    };
    let pal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            preview_title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let pal_inner = pal_block.inner(chunks[8]);
    f.render_widget(pal_block, chunks[8]);

    let pal_lines = vec![
        Line::from(vec![
            Span::styled("  ■ Primary: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("■■■■■  ", Style::default().fg(theme.primary)),
            Span::styled("  ■ Secondary: ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled("■■■■■  ", Style::default().fg(theme.secondary)),
            Span::styled("  ■ Accent: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("■■■■■", Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled(format!("  • 테마 명칭: {} ({:?})", app.theme.mode.label(lang), app.theme.mode), Style::default().fg(theme.text_highlight)),
        ]),
        Line::from(vec![
            Span::styled("  • 단축키: [c] 다음 테마   [C/p] 이전 테마   [↑/↓] 카테고리 전환   [T] 37개 테마 상세 모달", Style::default().fg(theme.text_dim)),
        ]),
    ];
    f.render_widget(Paragraph::new(pal_lines), pal_inner);

    // Config file hint (chunks[10])
    let cfg_hint = match lang {
        crate::i18n::Language::Ko => {
            "📁  설정 파일: ~/.config/rcal/theme.toml (색상 오버라이드 및 대비 조절 가능)"
        }
        crate::i18n::Language::Ja => {
            "📁  設定ファイル: ~/.config/rcal/theme.toml (色の上書き・コントラスト調整可能)"
        }
        _ => "📁  Config file: ~/.config/rcal/theme.toml (Color overrides & contrast options)",
    };
    f.render_widget(
        Paragraph::new(Span::styled(cfg_hint, Style::default().fg(theme.text_dim))),
        chunks[10],
    );

    selected_theme_rect.unwrap_or(row1_chunks[0])
}

// =============================================================================
// Tab 4: Language Selection Content
// =============================================================================
fn render_language_tab(f: &mut Frame, app: &mut App, area: Rect) -> Rect {
    let theme = app.theme;
    let lang = app.lang;
    let t = lang.t();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title & Current lang badge (y = inner.y + 3)
            Constraint::Length(1), // Instruction desc (y = inner.y + 4)
            Constraint::Length(1), // Spacer (y = inner.y + 5)
            Constraint::Length(3), // Info Banner Box (y = inner.y + 6..8)
            Constraint::Length(1), // Spacer (y = inner.y + 9)
            Constraint::Length(6), // 4 cols x 6 rows grid (y = inner.y + 10..15)
            Constraint::Length(1), // Spacer (y = inner.y + 16)
            Constraint::Length(1), // Current lang summary (y = inner.y + 17)
            Constraint::Min(0),
        ])
        .split(area);

    let is_focused = app.settings_section == SettingsSection::Language;
    let title_style = if is_focused {
        Style::default()
            .fg(theme.text_highlight)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.secondary)
            .add_modifier(Modifier::BOLD)
    };

    f.render_widget(
        Paragraph::new(Span::styled(
            format!("▶ {}", t.settings_lang_title),
            title_style,
        )),
        chunks[0],
    );

    let desc_str = match lang {
        crate::i18n::Language::Ko => {
            "방향키(↑↓←→ / hjkl)로 이동 후 [Enter/스페이스] 또는 클릭하여 언어를 즉시 변경합니다."
        }
        crate::i18n::Language::Ja => {
            "矢印キー(↑↓←→ / hjkl)で選択し、[Enter/スペース] またはクリックで言語を変更します。"
        }
        _ => "Navigate with arrow keys (↑↓←→ / hjkl) and press [Enter/Space] or click to select.",
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            desc_str,
            Style::default().fg(theme.text_muted),
        )),
        chunks[1],
    );

    // Info Banner Box (chunks[3])
    let banner_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " 🌐 다국어 지원 (21 Languages) ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let b_inner = banner_block.inner(chunks[3]);
    f.render_widget(banner_block, chunks[3]);

    let b_text = match lang {
        crate::i18n::Language::Ko => "전 세계 21개 주요 언어를 완벽 지원합니다. 언어를 변경하면 메뉴, 달력 표기, 상태 메시지가 실시간으로 업데이트됩니다.",
        crate::i18n::Language::Ja => "世界21言語を完全サポート。言語を変更するとメニューやカレンダー表記がリアルタイムに更新されます。",
        _ => "Fully supports 21 languages worldwide. Changing language updates all UI labels, dates, and alerts in real-time.",
    };
    f.render_widget(
        Paragraph::new(Span::styled(b_text, Style::default().fg(theme.text_dim))),
        b_inner,
    );

    // 4 cols x 6 rows grid (chunks[5]) (y = inner.y + 10..15)
    let all_langs = crate::i18n::Language::ALL;
    let grid_area = chunks[5];
    let num_cols = 4;
    let num_rows = (all_langs.len() + num_cols - 1) / num_cols;
    let col_w = (grid_area.width.saturating_sub(num_cols as u16 - 1)) / (num_cols as u16);

    let mut focused_rect = Rect::new(grid_area.x, grid_area.y, col_w, 1);

    for r in 0..num_rows {
        if r as u16 >= grid_area.height {
            break;
        }
        for c in 0..num_cols {
            let idx = r * num_cols + c;
            if idx >= all_langs.len() {
                break;
            }
            let lang_opt = all_langs[idx];
            let is_selected = app.lang == lang_opt;
            let is_cell_focused = idx == app.settings_lang_idx;
            let prefix = if is_selected { "● " } else { "○ " };
            let text = format!(" {}{}", prefix, lang_opt.label());

            let style = if is_selected && is_cell_focused {
                Style::default()
                    .fg(if theme.mode.is_light() {
                        Color::White
                    } else {
                        Color::Black
                    })
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else if is_selected {
                Style::default()
                    .fg(if theme.mode.is_light() {
                        Color::White
                    } else {
                        Color::Black
                    })
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else if is_cell_focused {
                Style::default()
                    .fg(theme.text_highlight)
                    .bg(theme.button_bg)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(theme.text_normal).bg(theme.button_bg)
            };

            let x = grid_area.x + (c as u16) * (col_w + 1);
            let y = grid_area.y + (r as u16);
            let w = if c == num_cols - 1 {
                grid_area.width.saturating_sub((c as u16) * (col_w + 1))
            } else {
                col_w
            };
            let rect = Rect {
                x,
                y,
                width: w,
                height: 1,
            };

            if is_cell_focused {
                focused_rect = rect;
            }

            let p = Paragraph::new(Line::from(vec![Span::raw(text)])).style(style);
            f.render_widget(p, rect);
            app.hit_map
                .register(rect, HitTarget::ModalSelectLanguage(lang_opt));
        }
    }

    // Summary line (chunks[7])
    let summary_text = format!(
        "  현재 선택된 언어: {} ({})",
        app.lang.label(),
        app.lang.code()
    );
    f.render_widget(
        Paragraph::new(Span::styled(
            summary_text,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        chunks[7],
    );

    focused_rect
}
