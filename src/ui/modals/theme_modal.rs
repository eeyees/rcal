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
    let popup_width = 90.min(area.width.saturating_sub(2));
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
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.text_highlight)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line & current theme badge
            Constraint::Length(1), // Spacer
            Constraint::Min(14),   // Main body: Left Shortcuts/Category menu + Right Theme Grid
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Config file hint
            Constraint::Length(1), // Close button
        ])
        .split(inner);

    // =========================================================================
    // 1. Current Theme Badge (Live Preview)
    // =========================================================================
    let all = ThemeMode::all();
    let curr_idx = all.iter().position(|m| *m == app.theme.mode).unwrap_or(0);
    let desc_str = match lang {
        crate::i18n::Language::Ko => "현재 테마 (실시간 미리보기):",
        crate::i18n::Language::Ja => "現在のテーマ (リアルタイム確認):",
        _ => "Current Theme (Live Preview):",
    };
    let current_badge = if state.original_mode != app.theme.mode {
        let orig_label = state.original_mode.label(lang);
        match lang {
            crate::i18n::Language::Ko => format!(
                " {} [ {}/{} ] {}  (원래 테마: {})",
                desc_str,
                curr_idx + 1,
                all.len(),
                app.theme.mode.label(lang),
                orig_label
            ),
            crate::i18n::Language::Ja => format!(
                " {} [ {}/{} ] {}  (元のテーマ: {})",
                desc_str,
                curr_idx + 1,
                all.len(),
                app.theme.mode.label(lang),
                orig_label
            ),
            _ => format!(
                " {} [ {}/{} ] {}  (Original: {})",
                desc_str,
                curr_idx + 1,
                all.len(),
                app.theme.mode.label(lang),
                orig_label
            ),
        }
    } else {
        format!(
            " {} [ {}/{} ] {}",
            desc_str,
            curr_idx + 1,
            all.len(),
            app.theme.mode.label(lang)
        )
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            current_badge,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )])),
        chunks[0],
    );

    // =========================================================================
    // 2. Main Body: Split into Left Menu Panel (Shortcuts & Category) + Right Theme Grid
    // =========================================================================
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(23), // Left: Shortcuts & Category Menu
            Constraint::Length(1),  // Spacer
            Constraint::Min(40),    // Right: Theme Grid (3 columns)
        ])
        .split(chunks[2]);

    // -------------------------------------------------------------------------
    // Left Menu Panel: Category filter tabs + Shortcuts guide
    // -------------------------------------------------------------------------
    let menu_title = match lang {
        crate::i18n::Language::Ko => " 단축키 & 분류 ",
        crate::i18n::Language::Ja => " 操作 & 分類 ",
        _ => " Shortcuts & Filter ",
    };
    let menu_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.panel_bg))
        .title(Span::styled(
            menu_title,
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        ));
    let menu_inner = menu_block.inner(body_chunks[0]);
    f.render_widget(menu_block, body_chunks[0]);

    let cat_filters = [
        (
            None,
            match lang {
                crate::i18n::Language::Ko => "0: 전체 (37)",
                crate::i18n::Language::Ja => "0: 全体 (37)",
                _ => "0: All (37)",
            },
        ),
        (
            Some(ThemeCategory::Dark),
            match lang {
                crate::i18n::Language::Ko => "1: 다크 (10)",
                crate::i18n::Language::Ja => "1: ダーク (10)",
                _ => "1: Dark (10)",
            },
        ),
        (
            Some(ThemeCategory::Nature),
            match lang {
                crate::i18n::Language::Ko => "2: 내추럴 (9)",
                crate::i18n::Language::Ja => "2: 自然 (9)",
                _ => "2: Nature (9)",
            },
        ),
        (
            Some(ThemeCategory::Vibrant),
            match lang {
                crate::i18n::Language::Ko => "3: 비비드 (8)",
                crate::i18n::Language::Ja => "3: ビビッド (8)",
                _ => "3: Vibrant (8)",
            },
        ),
        (
            Some(ThemeCategory::Light),
            match lang {
                crate::i18n::Language::Ko => "4: 라이트 (10)",
                crate::i18n::Language::Ja => "4: ライト (10)",
                _ => "4: Light (10)",
            },
        ),
    ];

    // Render 5 categories in Left Menu with clean spacing
    for (idx, (cat_opt, label)) in cat_filters.iter().enumerate() {
        if idx as u16 >= menu_inner.height {
            break;
        }
        let is_active = state.category_filter == *cat_opt;
        let icon = if is_active { "●" } else { "○" };
        let text = format!(" {}  {}", icon, label);

        let style = if is_active {
            Style::default()
                .fg(if theme.mode.is_light() {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_normal)
        };

        let row_rect = Rect {
            x: menu_inner.x,
            y: menu_inner.y + idx as u16,
            width: menu_inner.width,
            height: 1,
        };

        let p = Paragraph::new(Line::from(vec![Span::raw(text)])).style(style);
        f.render_widget(p, row_rect);
        app.hit_map
            .register(row_rect, HitTarget::ModalThemeCategory(*cat_opt));
    }

    // Divider and Keyboard Shortcuts in Left Menu
    let divider_y = menu_inner.y + 5;
    if divider_y < menu_inner.y + menu_inner.height {
        let divider_line = Span::styled("─────────────────────", Style::default().fg(theme.border));
        f.render_widget(
            Paragraph::new(divider_line),
            Rect {
                x: menu_inner.x,
                y: divider_y,
                width: menu_inner.width,
                height: 1,
            },
        );
    }

    let shortcuts_guide: Vec<&str> = match lang {
        crate::i18n::Language::Ko => vec![
            "단축키 안내:",
            " • 0-4 : 카테고리",
            " • ↑↓←→: 실시간 미리보기",
            " • hjkl: 커서 이동",
            " • Tab : 순환 이동",
            " • Enter: 테마 확정",
            " • Esc/q: 취소(복원)",
        ],
        crate::i18n::Language::Ja => vec![
            "操作案内:",
            " • 0-4 : 分類切替",
            " • ↑↓←→: プレビュー",
            " • hjkl: 移動",
            " • Tab : 巡回",
            " • Enter: 確定",
            " • Esc/q: 取消(復元)",
        ],
        _ => vec![
            "Key Controls:",
            " • 0-4 : Filter",
            " • ↑↓←→: Live Preview",
            " • hjkl: Move",
            " • Tab : Cycle",
            " • Enter: Apply",
            " • Esc/q: Cancel(Revert)",
        ],
    };

    let start_shortcut_y = menu_inner.y + 6;
    for (s_idx, s_text) in shortcuts_guide.iter().enumerate() {
        let line_y = start_shortcut_y + s_idx as u16;
        if line_y >= menu_inner.y + menu_inner.height {
            break;
        }
        let style = if s_idx == 0 {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_dim)
        };
        let line_rect = Rect {
            x: menu_inner.x,
            y: line_y,
            width: menu_inner.width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                format!(" {}", s_text),
                style,
            )])),
            line_rect,
        );
    }

    // -------------------------------------------------------------------------
    // Right: Theme Grid (3 columns)
    // -------------------------------------------------------------------------
    let filtered_themes = state.filtered_themes();
    let grid_area = body_chunks[2];
    let num_cols = if grid_area.width >= 58 { 3 } else { 2 };
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
            let radio = if is_current {
                "●"
            } else if mode == state.original_mode {
                "◎"
            } else {
                "○"
            };
            let cursor = if is_focused { "▶" } else { " " };

            let name = mode.short_name(lang);

            // Format cell text with color palette swatches: [■■■■]
            let swatch4 = if mode.is_light() {
                sample.today_bg
            } else {
                sample.border_focused
            };

            let label_span = Span::styled(
                format!("{} {} {:<9}", cursor, radio, name),
                if is_focused {
                    Style::default().add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .fg(sample.primary)
                        .add_modifier(Modifier::BOLD)
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
                    .fg(if theme.mode.is_light() {
                        Color::White
                    } else {
                        Color::Black
                    })
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(theme.primary).bg(theme.button_bg)
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
            app.hit_map
                .register(rect, HitTarget::ModalSelectTheme(mode));
        }
    }

    // =========================================================================
    // 3. Config file hint & Close Button
    // =========================================================================
    let cfg_hint = match lang {
        crate::i18n::Language::Ko => {
            "📁  설정 파일: ~/.config/rcal/theme.toml (색상 오버라이드 및 고대비 조절 가능)"
        }
        crate::i18n::Language::Ja => {
            "📁  設定ファイル: ~/.config/rcal/theme.toml (色の上書き・コントラスト調整可能)"
        }
        _ => "📁  Config file: ~/.config/rcal/theme.toml (Color overrides & contrast options)",
    };
    f.render_widget(
        Paragraph::new(Span::styled(cfg_hint, Style::default().fg(theme.text_dim)))
            .alignment(Alignment::Center),
        chunks[4],
    );

    let close_btn = Paragraph::new(Line::from(vec![Span::raw(lang.btn_close())]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(close_btn, chunks[5]);
    app.hit_map.register(chunks[5], HitTarget::ModalClose);
}
