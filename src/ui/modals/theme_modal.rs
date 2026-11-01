use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, ThemeModalState};
use crate::mouse::HitTarget;
use crate::ui::theme::{Theme, ThemeCategory, ThemeMode};

pub fn render(f: &mut Frame, app: &mut App, state: &ThemeModalState, area: Rect) {
    let popup_width = 86.min(area.width.saturating_sub(4));
    let popup_height = 23.min(area.height.saturating_sub(2));

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

    let title = lang.modal_theme_title();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.panel_bg))
        .title(Span::styled(title, Style::default().fg(theme.text_highlight).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line & current theme
            Constraint::Length(1), // Category Filter tabs (All, Dark, Nature, Vibrant, Light)
            Constraint::Length(1), // Spacer
            Constraint::Min(12),   // 3-column Theme Grid
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Config file hint
            Constraint::Length(1), // Keyboard shortcuts guide
            Constraint::Length(1), // Close button
        ])
        .split(inner);

    // =========================================================================
    // 1. Current Theme Badge
    // =========================================================================
    let all = ThemeMode::all();
    let curr_idx = all.iter().position(|m| *m == app.theme.mode).unwrap_or(0);
    let desc_str = match lang {
        crate::i18n::Language::Ko => "현재 테마:",
        crate::i18n::Language::Ja => "現在のテーマ:",
        _ => "Current Theme:",
    };
    let current_badge = format!(" {} [ {}/{} ] {}", desc_str, curr_idx + 1, all.len(), app.theme.mode.label(lang));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(current_badge, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ])),
        chunks[0],
    );

    // =========================================================================
    // 2. Category Filter Tabs
    // =========================================================================
    let cat_filters = [
        (None, match lang {
            crate::i18n::Language::Ko => "0:전체 (37)",
            crate::i18n::Language::Ja => "0:全テーマ (37)",
            _ => "0:All (37)",
        }),
        (Some(ThemeCategory::Dark), match lang {
            crate::i18n::Language::Ko => "1:다크 (10)",
            crate::i18n::Language::Ja => "1:ダーク (10)",
            _ => "1:Dark (10)",
        }),
        (Some(ThemeCategory::Nature), match lang {
            crate::i18n::Language::Ko => "2:내추럴 (9)",
            crate::i18n::Language::Ja => "2:自然 (9)",
            _ => "2:Nature (9)",
        }),
        (Some(ThemeCategory::Vibrant), match lang {
            crate::i18n::Language::Ko => "3:비비드 (8)",
            crate::i18n::Language::Ja => "3:ビビッド (8)",
            _ => "3:Vibrant (8)",
        }),
        (Some(ThemeCategory::Light), match lang {
            crate::i18n::Language::Ko => "4:라이트 (10)",
            crate::i18n::Language::Ja => "4:ライト (10)",
            _ => "4:Light (10)",
        }),
    ];

    let filter_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14), // All
            Constraint::Length(1),
            Constraint::Length(14), // Dark
            Constraint::Length(1),
            Constraint::Length(14), // Nature
            Constraint::Length(1),
            Constraint::Length(14), // Vibrant
            Constraint::Length(1),
            Constraint::Length(14), // Light
            Constraint::Min(0),
        ])
        .split(chunks[1]);

    for (idx, (cat_opt, label)) in cat_filters.iter().enumerate() {
        let is_active = state.category_filter == *cat_opt;
        let prefix = if is_active { "● " } else { "○ " };
        let text = format!("{}{}", prefix, label);

        let style = if is_active {
            Style::default()
                .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            theme.button_style()
        };

        let rect = filter_chunks[idx * 2];
        let p = Paragraph::new(Line::from(vec![Span::raw(text)]))
            .style(style)
            .alignment(Alignment::Center);
        f.render_widget(p, rect);
        app.hit_map.register(rect, HitTarget::ModalThemeCategory(*cat_opt));
    }

    // =========================================================================
    // 3. Filtered Themes Grid (3 columns)
    // =========================================================================
    let filtered_themes = state.filtered_themes();
    let grid_area = chunks[3];
    let num_cols = 3;
    let num_rows = (filtered_themes.len() + num_cols - 1) / num_cols;
    let col_w = (grid_area.width.saturating_sub(num_cols as u16 - 1)) / (num_cols as u16);

    for r in 0..num_rows {
        if r as u16 >= grid_area.height {
            break;
        }
        for c in 0..num_cols {
            let idx = r * num_cols + c;
            if idx >= filtered_themes.len() {
                break;
            }
            let mode = filtered_themes[idx];
            let is_current = app.theme.mode == mode;
            let is_focused = state.selected_idx == idx;

            let sample = Theme::from_mode(mode);
            let radio = if is_current { "●" } else { "○" };
            let cursor = if is_focused { "▶" } else { " " };

            let name = mode.short_name(lang);

            // Format cell text with color palette swatches: [■■■■]
            let swatch4 = if mode.is_light() { sample.today_bg } else { sample.border_focused };

            let label_span = Span::styled(
                format!("{} {} {:<9}", cursor, radio, name),
                if is_focused {
                    Style::default().add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default().fg(sample.primary).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_normal)
                },
            );

            let swatches = vec![
                Span::raw(" "),
                Span::styled("■", Style::default().fg(sample.primary)),
                Span::styled("■", Style::default().fg(sample.secondary)),
                Span::styled("■", Style::default().fg(sample.accent)),
                Span::styled("■", Style::default().fg(swatch4)),
            ];

            let mut spans = vec![label_span];
            spans.extend(swatches);

            let cell_style = if is_focused {
                Style::default()
                    .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default()
                    .fg(theme.primary)
                    .bg(theme.button_bg)
            } else {
                Style::default().bg(theme.panel_bg)
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

            let p = Paragraph::new(Line::from(spans)).style(cell_style);
            f.render_widget(p, rect);
            app.hit_map.register(rect, HitTarget::ModalSelectTheme(mode));
        }
    }

    // =========================================================================
    // 4. Config file hint & navigation instructions
    // =========================================================================
    let cfg_hint = match lang {
        crate::i18n::Language::Ko => "📁 설정 파일: ~/.config/rcal/theme.toml (색상 오버라이드 및 고대비 조절 가능)",
        crate::i18n::Language::Ja => "📁 設定ファイル: ~/.config/rcal/theme.toml (色の上書き・コントラスト調整可能)",
        _ => "📁 Config file: ~/.config/rcal/theme.toml (Color overrides & contrast options)",
    };
    f.render_widget(
        Paragraph::new(Span::styled(cfg_hint, Style::default().fg(theme.text_dim)))
            .alignment(Alignment::Center),
        chunks[5],
    );

    let hint_desc = match lang {
        crate::i18n::Language::Ko => "↑↓←→ / hjkl / Tab 이동 | [Enter/클릭] 선택 적용 | [0-4] 카테고리 | [Esc/q] 닫기",
        crate::i18n::Language::Ja => "↑↓←→ / hjkl / Tab 移動 | [Enter/クリック] 適用 | [0-4] カテゴリ | [Esc/q] 閉じる",
        _ => "↑↓←→ / hjkl / Tab navigate | [Enter/Click] apply | [0-4] category | [Esc/q] close",
    };
    f.render_widget(
        Paragraph::new(Span::styled(hint_desc, Style::default().fg(theme.text_muted)))
            .alignment(Alignment::Center),
        chunks[6],
    );

    // =========================================================================
    // 5. Close Button
    // =========================================================================
    let close_btn = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(close_btn, chunks[7]);
    app.hit_map.register(chunks[7], HitTarget::ModalClose);
}
