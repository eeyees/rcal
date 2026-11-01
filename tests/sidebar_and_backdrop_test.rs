#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::Terminal;
    use rcal::app::{ActiveModal, App};
    use rcal::events::handle_crossterm_event;
    use rcal::mouse::{HitMap, HitTarget};
    use rcal::storage::Database;
    use rcal::ui;

    #[test]
    fn test_sidebar_toggle_and_shortcuts() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        // 1. Initial state: sidebar is shown
        assert!(app.show_sidebar);

        // 2. toggle_sidebar method
        app.toggle_sidebar();
        assert!(!app.show_sidebar);
        assert!(app.get_active_status().unwrap().contains("사이드바"));

        // 3. Normal mode shortcut 'b'
        let event_b = CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        handle_crossterm_event(&mut app, event_b).unwrap();
        assert!(app.show_sidebar);

        // 4. Normal mode shortcut 'B' (Shift+B)
        let event_cap_b = CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT));
        handle_crossterm_event(&mut app, event_cap_b).unwrap();
        assert!(!app.show_sidebar);

        // 5. Normal mode shortcut Ctrl+B
        let event_ctrl_b = CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        handle_crossterm_event(&mut app, event_ctrl_b).unwrap();
        assert!(app.show_sidebar);

        // 6. Mouse click on BtnToggleSidebar
        app.hit_map.register(ratatui::layout::Rect::new(0, 0, 10, 1), HitTarget::BtnToggleSidebar);
        app.handle_mouse_click(5, 0).unwrap();
        assert!(!app.show_sidebar);
    }

    #[test]
    fn test_sidebar_layout_rendering_width() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        // Case A: Sidebar is visible
        app.show_sidebar = true;
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        // Hit map must contain sidebar targets (like BtnToggleSidebar and CalendarToggle)
        let has_toggle_sidebar_btn = app.hit_map.targets().iter().any(|(_, t)| *t == HitTarget::BtnToggleSidebar);
        assert!(has_toggle_sidebar_btn, "When sidebar is visible, BtnToggleSidebar must be registered");
        let has_cal_toggles = app.hit_map.targets().iter().any(|(_, t)| matches!(t, HitTarget::CalendarToggle(_)));
        assert!(has_cal_toggles, "When sidebar is visible, calendar toggles must be registered");

        // Month view cells should start after sidebar width (sidebar width >= 28)
        let month_cells: Vec<_> = app.hit_map.targets().iter().filter_map(|(rect, t)| {
            if matches!(t, HitTarget::MonthDay(_)) {
                Some(*rect)
            } else {
                None
            }
        }).collect();
        assert!(!month_cells.is_empty());
        let min_x_with_sidebar = month_cells.iter().map(|r| r.x).min().unwrap();
        assert!(min_x_with_sidebar >= 28, "Month view x offset must accommodate sidebar (got {})", min_x_with_sidebar);

        // Case B: Sidebar is hidden
        app.show_sidebar = false;
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        let has_toggle_sidebar_btn_hidden = app.hit_map.targets().iter().any(|(_, t)| *t == HitTarget::BtnToggleSidebar);
        assert!(!has_toggle_sidebar_btn_hidden, "When sidebar is hidden, BtnToggleSidebar should not be registered");
        let has_cal_toggles_hidden = app.hit_map.targets().iter().any(|(_, t)| matches!(t, HitTarget::CalendarToggle(_)));
        assert!(!has_cal_toggles_hidden, "When sidebar is hidden, calendar toggles should not be registered");

        // Month view cells now expand across full workspace (min x starts at 0)
        let month_cells_no_sidebar: Vec<_> = app.hit_map.targets().iter().filter_map(|(rect, t)| {
            if matches!(t, HitTarget::MonthDay(_)) {
                Some(*rect)
            } else {
                None
            }
        }).collect();
        let min_x_no_sidebar = month_cells_no_sidebar.iter().map(|r| r.x).min().unwrap();
        assert_eq!(min_x_no_sidebar, 0, "Month view must take full width starting at x=0 when sidebar is hidden");
    }

    #[test]
    fn test_modal_backdrop_click_and_clickthrough_prevention() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        // 1. Open Help Modal and render
        app.open_help_modal();
        assert!(matches!(app.modal, ActiveModal::Help));

        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        // Targets must have ModalBackdrop covering full screen, and ModalConsume covering dialog
        let backdrop_exists = app.hit_map.targets().iter().any(|(_, t)| *t == HitTarget::ModalBackdrop);
        assert!(backdrop_exists, "ModalBackdrop must be registered when modal is open");

        let consume_exists = app.hit_map.targets().iter().any(|(_, t)| *t == HitTarget::ModalConsume);
        assert!(consume_exists, "ModalConsume must be registered for modal dialog area");

        // 2. Click outside modal on backdrop (e.g. at (1, 1), which is header/backdrop area)
        let target_at_top = app.hit_map.check(1, 1);
        assert_eq!(target_at_top, Some(HitTarget::ModalBackdrop));
        app.handle_mouse_click(1, 1).unwrap();
        assert!(matches!(app.modal, ActiveModal::None), "Clicking on backdrop must safely close the modal");

        // 3. Open Settings Modal and click inside non-button dialog area (absorb click)
        app.open_settings_modal();
        assert!(matches!(app.modal, ActiveModal::Settings));
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();

        // Find rect of ModalConsume
        let consume_rect = app.hit_map.targets().iter().find_map(|(r, t)| {
            if *t == HitTarget::ModalConsume {
                Some(*r)
            } else {
                None
            }
        }).unwrap();

        // Inner title area inside dialog (consume_rect.x + 2, consume_rect.y + 1)
        let target_inside = app.hit_map.check(consume_rect.x + 2, consume_rect.y + 1);
        assert_eq!(target_inside, Some(HitTarget::ModalConsume));
        app.handle_mouse_click(consume_rect.x + 2, consume_rect.y + 1).unwrap();
        // Modal must remain open and not close or fall through
        assert!(matches!(app.modal, ActiveModal::Settings));

        // 4. Verification that background clicks NEVER execute while modal is active:
        // Even if a MonthDay target is tested directly while modal is set:
        let original_date = app.selected_date;
        let different_date = original_date + chrono::Duration::days(5);

        // Simulate a hypothetical hit map containing a MonthDay target
        let mut isolated_hit_map = HitMap::new();
        isolated_hit_map.register(ratatui::layout::Rect::new(10, 10, 10, 5), HitTarget::MonthDay(different_date));
        isolated_hit_map.register(ratatui::layout::Rect::new(0, 0, 120, 40), HitTarget::ModalBackdrop);

        // Under isolated_hit_map with modal active, clicking at (15, 12) must hit ModalBackdrop, NOT MonthDay
        assert_eq!(isolated_hit_map.check(15, 12), Some(HitTarget::ModalBackdrop));

        // And if has_modal is true, any target that is not a modal target is suppressed by HitMap::check
        let mut leak_test_map = HitMap::new();
        leak_test_map.register(ratatui::layout::Rect::new(10, 10, 10, 5), HitTarget::MonthDay(different_date));
        leak_test_map.register(ratatui::layout::Rect::new(50, 50, 20, 10), HitTarget::ModalClose);
        // Position (15, 12) is inside MonthDay, but leak_test_map has a modal target
        assert_eq!(leak_test_map.check(15, 12), None, "HitMap::check must return None instead of background targets when modal is present");
    }

    #[test]
    fn test_date_display_stability_on_focus() {
        let db = Database::in_memory().unwrap();
        let mut app = App::new(db).unwrap();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        // Select a specific date (e.g. 15th)
        let date_15 = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        let date_16 = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();

        app.selected_date = date_15;
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let buffer_with_15_focused = terminal.backend().buffer().clone();

        // Switch focus to 16th
        app.selected_date = date_16;
        terminal.draw(|f| ui::render(f, &mut app)).unwrap();
        let buffer_with_16_focused = terminal.backend().buffer().clone();

        // Helper to find column index of a string in a row
        let find_col = |buf: &Buffer, row: u16, search: &str| -> Option<u16> {
            let width = buf.area.width;
            let mut line = String::new();
            for x in 0..width {
                line.push_str(buf[(x, row)].symbol());
            }
            line.find(search).map(|pos| pos as u16)
        };

        // Find row where " 15 " appears
        let mut row_15 = None;
        for y in 0..40 {
            if find_col(&buffer_with_15_focused, y, " 15 ").is_some() {
                row_15 = Some(y);
                break;
            }
        }
        let y_15 = row_15.expect("Must find ' 15 ' in month view buffer");

        let col_when_focused = find_col(&buffer_with_15_focused, y_15, " 15 ").unwrap();
        let col_when_unfocused = find_col(&buffer_with_16_focused, y_15, " 15 ").unwrap();

        assert_eq!(
            col_when_focused, col_when_unfocused,
            "Date '15' horizontal column position must NOT shift when focused ({}) vs unfocused ({})!",
            col_when_focused, col_when_unfocused
        );

        // Also check single digit day (e.g. " 5 ")
        let mut row_5 = None;
        for y in 0..40 {
            if find_col(&buffer_with_15_focused, y, " 5 ").is_some() {
                row_5 = Some(y);
                break;
            }
        }
        let y_5 = row_5.expect("Must find ' 5 ' in month view buffer");
        let col_5_state1 = find_col(&buffer_with_15_focused, y_5, " 5 ").unwrap();
        let col_5_state2 = find_col(&buffer_with_16_focused, y_5, " 5 ").unwrap();

        assert_eq!(
            col_5_state1, col_5_state2,
            "Date '5' horizontal column position must NOT shift between selection states!"
        );
    }

    #[test]
    fn test_sidebar_theme_shortcut_and_interaction() -> anyhow::Result<()> {
        let db = Database::in_memory()?;
        let mut app = App::new(db)?;

        // Ensure sidebar tips and key_tips contain theme shortcuts
        let tips = app.lang.sidebar_tips();
        assert!(tips.iter().any(|t| t.contains("T 키") || t.contains("'T'") || t.contains("Tキー")), "sidebar_tips must contain T theme shortcut");
        let key_tips = app.lang.sidebar_key_tips();
        assert!(key_tips.iter().any(|k| k.contains("T:")), "sidebar_key_tips must contain T theme shortcut");

        // Render sidebar in terminal
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend)?;
        terminal.draw(|f| ui::render(f, &mut app))?;

        // Hit map must have registered HitTarget::BtnTheme from sidebar
        let has_theme_hit = app.hit_map.targets().iter().any(|(_, t)| *t == HitTarget::BtnTheme);
        assert!(has_theme_hit, "Sidebar should register BtnTheme hit target for mouse clicking theme shortcut");

        // Find the rect for BtnTheme that is in the sidebar (x < 35, y > 2)
        let sidebar_theme_hit = app.hit_map.targets().iter().find(|(r, t)| *t == HitTarget::BtnTheme && r.x < 35 && r.y > 2);
        assert!(sidebar_theme_hit.is_some(), "Sidebar must have a clickable BtnTheme target");

        let (rect, _) = sidebar_theme_hit.unwrap();
        app.handle_mouse_click(rect.x + 1, rect.y)?;
        assert!(matches!(app.modal, ActiveModal::Theme(_)), "Clicking theme shortcut in sidebar must open Theme modal");

        Ok(())
    }
}
