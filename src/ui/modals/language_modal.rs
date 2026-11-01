use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, LanguageModalState};
use crate::i18n::Language;
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App, state: &LanguageModalState, area: Rect) {
    let popup_width = 74.min(area.width.saturating_sub(4));
    let popup_height = 15.min(area.height.saturating_sub(2));

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

    let title = match lang {
        Language::Ko => " 🌐 언어 선택 (Language Selection) ",
        Language::Ja => " 🌐 言語の選択 (Language Selection) ",
        Language::ZhCn => " 🌐 语言选择 (Language Selection) ",
        Language::ZhTw => " 🌐 語言選擇 (Language Selection) ",
        _ => " 🌐 Select Language ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(title, Style::default().fg(theme.text_highlight).add_modifier(Modifier::BOLD)));
    let inner = block.inner(popup_rect);
    f.render_widget(block, popup_rect);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Description / navigation guide
            Constraint::Length(1), // Spacer
            Constraint::Length(7), // 3 columns x 7 rows grid
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Close button
        ])
        .split(inner);

    // Guide text
    let hint_desc = match lang {
        Language::Ko => "↑↓←→ / hjkl / Tab 이동, [Enter/클릭] 선택, [Esc/q] 닫기",
        Language::Ja => "↑↓←→ / hjkl / Tab 移動、[Enter/クリック] 選択、[Esc/q] 閉じる",
        Language::ZhCn => "↑↓←→ / hjkl / Tab 移动, [Enter/点击] 选择, [Esc/q] 关闭",
        Language::ZhTw => "↑↓←→ / hjkl / Tab 移動, [Enter/點擊] 選擇, [Esc/q] 關閉",
        _ => "↑↓←→ / hjkl / Tab navigate, [Enter/Click] select, [Esc/q] close",
    };
    f.render_widget(
        Paragraph::new(Span::styled(hint_desc, Style::default().fg(theme.text_muted)))
            .alignment(Alignment::Center),
        chunks[0],
    );

    // Grid of all 21 languages
    let all_langs = Language::ALL;
    let grid_area = chunks[2];
    let num_cols = 3;
    let num_rows = (all_langs.len() + num_cols - 1) / num_cols;
    let col_w = (grid_area.width.saturating_sub(num_cols as u16 - 1)) / (num_cols as u16);

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
            let is_current = app.lang == lang_opt;
            let is_focused = state.selected_idx == idx;

            let radio = if is_current { "●" } else { "○" };
            let cursor = if is_focused { "▶" } else { " " };
            let text = format!("{} {} {}", cursor, radio, lang_opt.label());

            let style = if is_focused {
                Style::default()
                    .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default()
                    .fg(if theme.mode.is_light() { Color::White } else { Color::Black })
                    .bg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.text_normal)
                    .bg(theme.button_bg)
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

            let p = Paragraph::new(Line::from(vec![Span::raw(text)])).style(style);
            f.render_widget(p, rect);
            app.hit_map.register(rect, HitTarget::ModalSelectLanguage(lang_opt));

            if is_focused {
                f.set_cursor_position((x + 1, y));
            }
        }
    }

    // Close button
    let close_text = lang.btn_close();
    let btn_close = Paragraph::new(Line::from(vec![Span::raw(close_text)]))
        .style(theme.button_style())
        .alignment(Alignment::Center);
    f.render_widget(btn_close, chunks[4]);
    app.hit_map.register(chunks[4], HitTarget::ModalClose);
}
