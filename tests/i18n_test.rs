#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rcal::app::{ActiveModal, App};
    use rcal::i18n::Language;
    use rcal::model::WeekStartDay;
    use rcal::mouse::HitTarget;
    use rcal::storage::Database;

    #[test]
    fn test_i18n_toggle_and_labels() {
        let mut curr = Language::Ko;
        for expected in Language::ALL {
            assert_eq!(curr, *expected);
            assert!(!curr.label().is_empty());
            assert!(!curr.short_label().is_empty());
            assert!(!curr.view_month().is_empty());
            assert!(!curr.view_week().is_empty());
            assert!(!curr.btn_new_event().is_empty());
            assert!(!curr.btn_sync().is_empty());
            curr = curr.toggle();
        }
        // After full cycle through all 21 languages, it wraps back to Ko
        assert_eq!(curr, Language::Ko);

        let lang_ko = Language::Ko;
        let lang_en = Language::En;
        let lang_ja = Language::Ja;
        let lang_zh_cn = Language::ZhCn;
        let lang_zh_tw = Language::ZhTw;
        let lang_mn = Language::Mn;

        assert_eq!(lang_ko.label(), "한국어");
        assert_eq!(lang_en.label(), "English");
        assert_eq!(lang_ja.label(), "日本語");
        assert_eq!(lang_zh_cn.label(), "简体中文");
        assert_eq!(lang_zh_tw.label(), "繁體中文");
        assert_eq!(lang_mn.label(), "Монгол");

        assert_eq!(lang_ko.short_label(), "한");
        assert_eq!(lang_en.short_label(), "EN");
        assert_eq!(lang_ja.short_label(), "日");
        assert_eq!(lang_zh_cn.short_label(), "简");
        assert_eq!(lang_zh_tw.short_label(), "繁");
        assert_eq!(lang_mn.short_label(), "MN");

        // View names
        assert_eq!(lang_ko.view_month(), "1:월간");
        assert_eq!(lang_en.view_month(), "1:Month");
        assert_eq!(lang_ja.view_month(), "1:月間");
        assert_eq!(lang_zh_cn.view_month(), "1:月视图");
        assert_eq!(lang_zh_tw.view_month(), "1:月檢視");
        assert_eq!(lang_mn.view_month(), "1:Сар");

        assert_eq!(lang_ko.view_week(), "2:주간");
        assert_eq!(lang_en.view_week(), "2:Week");
        assert_eq!(lang_ja.view_week(), "2:週間");
        assert_eq!(lang_zh_cn.view_week(), "2:周视图");
        assert_eq!(lang_zh_tw.view_week(), "2:週檢視");
        assert_eq!(lang_mn.view_week(), "2:Долоо хоног");

        // Action buttons
        assert_eq!(lang_ko.btn_new_event(), "+ 새 일정");
        assert_eq!(lang_en.btn_new_event(), "+ New");
        assert_eq!(lang_ja.btn_new_event(), "+ 新規予定");
        assert_eq!(lang_zh_cn.btn_new_event(), "+ 新建");
        assert_eq!(lang_zh_tw.btn_new_event(), "+ 新增");
        assert_eq!(lang_mn.btn_new_event(), "+ Шинэ");

        assert_eq!(lang_ko.btn_sync(), "⟳  동기화");
        assert_eq!(lang_en.btn_sync(), "⟳  Sync");
        assert_eq!(lang_ja.btn_sync(), "⟳  同期");
        assert_eq!(lang_zh_cn.btn_sync(), "⟳ 同步");
        assert_eq!(lang_zh_tw.btn_sync(), "⟳ 同步");
        assert_eq!(lang_mn.btn_sync(), "⟳ Синк");

        assert_eq!(lang_ko.btn_accounts(), "☁  계정연동");
        assert_eq!(lang_en.btn_accounts(), "☁  Accounts");
        assert_eq!(lang_ja.btn_accounts(), "☁  アカウント");

        assert_eq!(lang_ko.btn_settings(), "⚙  설정");
        assert_eq!(lang_en.btn_settings(), "⚙  Setup");
        assert_eq!(lang_ja.btn_settings(), "⚙  設定");

        assert_eq!(lang_ko.btn_theme(), "🎨  테마");
        assert_eq!(lang_en.btn_theme(), "🎨  Theme");
        assert_eq!(lang_ja.btn_theme(), "🎨  テーマ");
    }

    #[test]
    fn test_i18n_date_formatting() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();

        let ko_month_year = Language::Ko.format_month_year(date);
        assert_eq!(ko_month_year, "2026년 10월");

        let en_month_year = Language::En.format_month_year(date);
        assert_eq!(en_month_year, "October 2026");

        let ja_month_year = Language::Ja.format_month_year(date);
        assert_eq!(ja_month_year, "2026年 10月");

        let ko_headers = Language::Ko.weekday_headers();
        assert_eq!(ko_headers, ["일", "월", "화", "수", "목", "금", "토"]);

        let en_headers = Language::En.weekday_headers();
        assert_eq!(en_headers, ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]);

        let ja_headers = Language::Ja.weekday_headers();
        assert_eq!(ja_headers, ["日", "月", "火", "水", "木", "金", "土"]);

        // Long date formatting
        assert!(Language::Ko.format_date_long(date).contains("2026년 10월 30일"));
        assert!(Language::En.format_date_long(date).contains("October 30, 2026"));
        assert!(Language::Ja.format_date_long(date).contains("2026年 10月 30日"));
    }

    #[test]
    fn test_all_languages_have_complete_translations() {
        for lang in Language::ALL {
            let t = lang.t();
            // Metadata
            assert!(!t.code.is_empty(), "Language {:?} has empty code", lang);
            assert!(!t.label.is_empty(), "Language {:?} has empty label", lang);
            assert!(!t.short_label.is_empty(), "Language {:?} has empty short_label", lang);

            // Views
            assert!(!t.view_month.is_empty());
            assert!(!t.view_week.is_empty());
            assert!(!t.view_day.is_empty());
            assert!(!t.view_agenda.is_empty());

            // Buttons
            assert!(!t.btn_today.is_empty());
            assert!(!t.btn_new_event.is_empty());
            assert!(!t.btn_sync.is_empty());
            assert!(!t.btn_save.is_empty());
            assert!(!t.btn_cancel.is_empty());
            assert!(!t.btn_close.is_empty());

            // Modals
            assert!(!t.modal_new_event.is_empty());
            assert!(!t.modal_edit_event.is_empty());
            assert!(!t.modal_settings_title.is_empty());
            assert!(!t.modal_help_title.is_empty());
            assert!(!t.modal_cal_create_title.is_empty());
            assert!(!t.modal_account_add_title.is_empty());

            // Status messages
            assert!(!t.msg_welcome.is_empty());
            assert!(!t.msg_saved.is_empty());
            assert!(!t.msg_deleted.is_empty());

            // Weekday headers count
            assert_eq!(t.weekday_headers.len(), 7);
            assert_eq!(t.week_start_sun_headers.len(), 7);
            assert_eq!(t.week_start_mon_headers.len(), 7);
            assert_eq!(t.week_start_sat_headers.len(), 7);
        }
    }

    #[test]
    fn test_language_code_parsing() {
        assert_eq!(Language::from_code("ko"), Some(Language::Ko));
        assert_eq!(Language::from_code("ko_kr"), Some(Language::Ko));
        assert_eq!(Language::from_code("KO-KR"), Some(Language::Ko));
        assert_eq!(Language::from_code("ko_KR.UTF-8"), Some(Language::Ko));
        assert_eq!(Language::from_code("korean"), Some(Language::Ko));

        assert_eq!(Language::from_code("en"), Some(Language::En));
        assert_eq!(Language::from_code("en_us"), Some(Language::En));
        assert_eq!(Language::from_code("EN-US"), Some(Language::En));
        assert_eq!(Language::from_code("en_US.UTF-8"), Some(Language::En));
        assert_eq!(Language::from_code("en-gb"), Some(Language::En));
        assert_eq!(Language::from_code("en-ca"), Some(Language::En));
        assert_eq!(Language::from_code("en-au"), Some(Language::En));
        assert_eq!(Language::from_code("english"), Some(Language::En));

        assert_eq!(Language::from_code("ja"), Some(Language::Ja));
        assert_eq!(Language::from_code("ja_jp"), Some(Language::Ja));
        assert_eq!(Language::from_code("JA-JP"), Some(Language::Ja));
        assert_eq!(Language::from_code("ja_JP.UTF-8"), Some(Language::Ja));
        assert_eq!(Language::from_code("japanese"), Some(Language::Ja));

        // Additional 18 languages
        assert_eq!(Language::from_code("zh-cn"), Some(Language::ZhCn));
        assert_eq!(Language::from_code("zh_cn"), Some(Language::ZhCn));
        assert_eq!(Language::from_code("zh"), Some(Language::ZhCn));
        assert_eq!(Language::from_code("zh-Hans-CN"), Some(Language::ZhCn));
        assert_eq!(Language::from_code("zh-tw"), Some(Language::ZhTw));
        assert_eq!(Language::from_code("zh_tw"), Some(Language::ZhTw));
        assert_eq!(Language::from_code("zh-Hant-TW"), Some(Language::ZhTw));
        assert_eq!(Language::from_code("zh-HK"), Some(Language::ZhTw));
        assert_eq!(Language::from_code("mn"), Some(Language::Mn));
        assert_eq!(Language::from_code("es"), Some(Language::Es));
        assert_eq!(Language::from_code("es-MX"), Some(Language::Es));
        assert_eq!(Language::from_code("fr"), Some(Language::Fr));
        assert_eq!(Language::from_code("fr-CA"), Some(Language::Fr));
        assert_eq!(Language::from_code("de"), Some(Language::De));
        assert_eq!(Language::from_code("de-AT"), Some(Language::De));
        assert_eq!(Language::from_code("ru"), Some(Language::Ru));
        assert_eq!(Language::from_code("pt"), Some(Language::Pt));
        assert_eq!(Language::from_code("pt-BR"), Some(Language::Pt));
        assert_eq!(Language::from_code("it"), Some(Language::It));
        assert_eq!(Language::from_code("it-CH"), Some(Language::It));
        assert_eq!(Language::from_code("vi"), Some(Language::Vi));
        assert_eq!(Language::from_code("id"), Some(Language::Id));
        assert_eq!(Language::from_code("th"), Some(Language::Th));
        assert_eq!(Language::from_code("ar"), Some(Language::Ar));
        assert_eq!(Language::from_code("hi"), Some(Language::Hi));
        assert_eq!(Language::from_code("tr"), Some(Language::Tr));
        assert_eq!(Language::from_code("nl"), Some(Language::Nl));
        assert_eq!(Language::from_code("pl"), Some(Language::Pl));
        assert_eq!(Language::from_code("sv"), Some(Language::Sv));

        // Unsupported languages return None
        assert_eq!(Language::from_code("fi"), None);
        assert_eq!(Language::from_code("fi-FI"), None);
        assert_eq!(Language::from_code("cs-CZ"), None);
        assert_eq!(Language::from_code("da-DK"), None);
        assert_eq!(Language::from_code("el-GR"), None);
        assert_eq!(Language::from_code("C"), None);
        assert_eq!(Language::from_code("POSIX"), None);
        assert_eq!(Language::from_code("invalid_xyz"), None);
        assert_eq!(Language::from_code(""), None);
    }

    #[test]
    fn test_system_language_detection_and_fallback() -> anyhow::Result<()> {
        // Supported locales detected properly
        assert_eq!(Language::detect_from_locales(["ko-KR"]), Language::Ko);
        assert_eq!(Language::detect_from_locales(["en-US"]), Language::En);
        assert_eq!(Language::detect_from_locales(["ja-JP"]), Language::Ja);
        assert_eq!(Language::detect_from_locales(["zh-Hans-CN"]), Language::ZhCn);
        assert_eq!(Language::detect_from_locales(["zh-Hant-TW"]), Language::ZhTw);
        assert_eq!(Language::detect_from_locales(["es-MX"]), Language::Es);
        assert_eq!(Language::detect_from_locales(["fr-FR"]), Language::Fr);
        assert_eq!(Language::detect_from_locales(["de-DE"]), Language::De);

        // Unsupported locales fallback to English
        assert_eq!(Language::detect_from_locales(["fi-FI"]), Language::En);
        assert_eq!(Language::detect_from_locales(["cs-CZ"]), Language::En);
        assert_eq!(Language::detect_from_locales(["da-DK"]), Language::En);
        assert_eq!(Language::detect_from_locales(["el-GR"]), Language::En);
        assert_eq!(Language::detect_from_locales(["hu-HU"]), Language::En);
        assert_eq!(Language::detect_from_locales(["invalid_lang_code"]), Language::En);
        assert_eq!(Language::detect_from_locales(&[] as &[&str]), Language::En);

        // Multiple preferred locales: skips unsupported and picks first supported
        assert_eq!(
            Language::detect_from_locales(["fi-FI", "de-DE"]),
            Language::De
        );
        assert_eq!(
            Language::detect_from_locales(["cs-CZ", "el-GR"]),
            Language::En
        );

        // App startup with no DB setting follows system detection
        let db = Database::in_memory()?;
        let app = App::new(db)?;
        assert_eq!(app.lang, Language::detect());

        // App startup with DB setting follows DB setting, overriding system
        let db_configured = Database::in_memory()?;
        db_configured.set_setting("language", "ja")?;
        let app_configured = App::new(db_configured)?;
        assert_eq!(app_configured.lang, Language::Ja);

        Ok(())
    }

    #[test]
    fn test_language_db_persistence_and_mouse_selection() -> anyhow::Result<()> {
        let db_path = std::env::temp_dir().join(format!("test_i18n_{}.db", uuid::Uuid::new_v4()));
        let db = Database::new(Some(&db_path))?;
        let mut app = App::new(db)?;
        app.set_language(Language::Ko);

        assert_eq!(app.lang, Language::Ko);

        // Toggle language: Ko -> En
        app.toggle_language();
        assert_eq!(app.lang, Language::En);
        assert_eq!(app.db.get_setting("language")?, Some("en".to_string()));

        // Toggle language: En -> Ja
        app.toggle_language();
        assert_eq!(app.lang, Language::Ja);
        assert_eq!(app.db.get_setting("language")?, Some("ja".to_string()));

        // Toggle language: Ja -> ZhCn
        app.toggle_language();
        assert_eq!(app.lang, Language::ZhCn);
        assert_eq!(app.db.get_setting("language")?, Some("zh-cn".to_string()));
        assert!(app.get_active_status().unwrap().contains("简体中文"));

        // Simulate mouse click on Spanish option in settings modal
        app.open_settings_modal();
        assert_eq!(app.modal, ActiveModal::Settings);
        app.hit_map.register(
            ratatui::layout::Rect::new(40, 10, 20, 1),
            HitTarget::ModalSelectLanguage(Language::Es),
        );
        app.handle_mouse_click(45, 10)?;
        assert_eq!(app.lang, Language::Es);
        assert_eq!(app.db.get_setting("language")?, Some("es".to_string()));

        // Reopen app with same database: should restore Spanish
        let db2 = Database::new(Some(&db_path))?;
        let app2 = App::new(db2)?;
        assert_eq!(app2.lang, Language::Es);

        let _ = std::fs::remove_file(&db_path);
        Ok(())
    }

    #[test]
    fn test_language_keyboard_shortcuts() -> anyhow::Result<()> {
        use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
        use rcal::events::handle_crossterm_event;

        let db = Database::in_memory()?;
        let mut app = App::new(db)?;
        app.set_language(Language::Ko);
        let initial_date = app.selected_date;

        // In normal mode:
        // 'l' (lowercase) is navigation right, not language modal
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
        )?;
        assert_eq!(app.selected_date, initial_date + chrono::Duration::days(1));
        assert_eq!(app.lang, Language::Ko);
        assert_eq!(app.modal, ActiveModal::None);

        // 'L' (Shift+l) opens Language selection modal
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::SHIFT)),
        )?;
        assert!(matches!(app.modal, ActiveModal::Language(_)));

        // In Language modal: navigate Right to English (idx 1), then Right to Japanese (idx 2)
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
        )?;
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
        )?;
        // Press Enter to select Japanese
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        )?;
        assert_eq!(app.lang, Language::Ja);
        assert_eq!(app.modal, ActiveModal::None);

        // 'ㅣ' (Korean IME without Shift) normalizes to 'L': opens Language modal
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('ㅣ'), KeyModifiers::NONE)),
        )?;
        assert!(matches!(app.modal, ActiveModal::Language(_)));
        app.close_modal();

        // Shift + 'ㅣ' (Korean IME with Shift) also normalizes to 'L': opens Language modal
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('ㅣ'), KeyModifiers::SHIFT)),
        )?;
        assert!(matches!(app.modal, ActiveModal::Language(_)));

        // Press 'j' (Down) in 3-column grid: moves from idx 2 (Ja) to idx 5 (Mn)
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
        )?;
        // Press Space to select Mongolian
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
        )?;
        assert_eq!(app.lang, Language::Mn);
        assert_eq!(app.modal, ActiveModal::None);

        // In Settings modal:
        app.open_settings_modal();
        assert_eq!(app.modal, ActiveModal::Settings);

        // In Settings modal, 'l' or 'L' opens Language modal
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
        )?;
        assert!(matches!(app.modal, ActiveModal::Language(_)));

        // Press 'Esc' to cancel without change
        handle_crossterm_event(
            &mut app,
            CrosstermEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        )?;
        assert_eq!(app.lang, Language::Mn);
        assert_eq!(app.modal, ActiveModal::None);

        Ok(())
    }

    #[test]
    fn test_week_start_labels_and_headers_per_language() {
        for lang in Language::ALL {
            let sun_headers = lang.weekday_headers_for(WeekStartDay::Sunday);
            let mon_headers = lang.weekday_headers_for(WeekStartDay::Monday);
            let sat_headers = lang.weekday_headers_for(WeekStartDay::Saturday);

            assert_eq!(sun_headers.len(), 7);
            assert_eq!(mon_headers.len(), 7);
            assert_eq!(sat_headers.len(), 7);

            let label_sun = lang.week_start_label(WeekStartDay::Sunday);
            let label_mon = lang.week_start_label(WeekStartDay::Monday);
            let label_sat = lang.week_start_label(WeekStartDay::Saturday);

            assert!(!label_sun.is_empty());
            assert!(!label_mon.is_empty());
            assert!(!label_sat.is_empty());
        }
    }
}
