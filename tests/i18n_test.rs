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
        let lang_ko = Language::Ko;
        let lang_en = lang_ko.toggle();
        let lang_ja = lang_en.toggle();
        let back_to_ko = lang_ja.toggle();

        assert_eq!(lang_en, Language::En);
        assert_eq!(lang_ja, Language::Ja);
        assert_eq!(back_to_ko, Language::Ko);

        assert_eq!(lang_ko.label(), "한국어");
        assert_eq!(lang_en.label(), "English");
        assert_eq!(lang_ja.label(), "日本語");

        assert_eq!(lang_ko.short_label(), "한");
        assert_eq!(lang_en.short_label(), "EN");
        assert_eq!(lang_ja.short_label(), "日");

        // View names
        assert_eq!(lang_ko.view_month(), "1:월간");
        assert_eq!(lang_en.view_month(), "1:Month");
        assert_eq!(lang_ja.view_month(), "1:月間");
        assert_eq!(lang_ko.view_week(), "2:주간");
        assert_eq!(lang_en.view_week(), "2:Week");
        assert_eq!(lang_ja.view_week(), "2:週間");

        // Action buttons
        assert_eq!(lang_ko.btn_new_event(), "+ 새 일정");
        assert_eq!(lang_en.btn_new_event(), "+ New");
        assert_eq!(lang_ja.btn_new_event(), "+ 新規予定");
        assert_eq!(lang_ko.btn_sync(), "⟳ 동기화");
        assert_eq!(lang_en.btn_sync(), "⟳ Sync");
        assert_eq!(lang_ja.btn_sync(), "⟳ 同期");
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
        assert_eq!(Language::from_code("korean"), Some(Language::Ko));

        assert_eq!(Language::from_code("en"), Some(Language::En));
        assert_eq!(Language::from_code("en_us"), Some(Language::En));
        assert_eq!(Language::from_code("EN-US"), Some(Language::En));
        assert_eq!(Language::from_code("english"), Some(Language::En));

        assert_eq!(Language::from_code("ja"), Some(Language::Ja));
        assert_eq!(Language::from_code("ja_jp"), Some(Language::Ja));
        assert_eq!(Language::from_code("JA-JP"), Some(Language::Ja));
        assert_eq!(Language::from_code("japanese"), Some(Language::Ja));

        assert_eq!(Language::from_code("invalid_xyz"), None);
    }

    #[test]
    fn test_language_db_persistence_and_mouse_selection() -> anyhow::Result<()> {
        let db_path = std::env::temp_dir().join(format!("test_i18n_{}.db", uuid::Uuid::new_v4()));
        let db = Database::new(Some(&db_path))?;
        let mut app = App::new(db)?;

        // Default language is Korean
        assert_eq!(app.lang, Language::Ko);

        // Switch to Japanese
        app.set_language(Language::Ja);
        assert_eq!(app.lang, Language::Ja);
        assert_eq!(app.db.get_setting("language")?, Some("ja".to_string()));
        assert!(app.get_active_status().unwrap().contains("日本語"));

        // Toggle language: Ja -> Ko
        app.toggle_language();
        assert_eq!(app.lang, Language::Ko);
        assert_eq!(app.db.get_setting("language")?, Some("ko".to_string()));

        // Toggle language again: Ko -> En
        app.toggle_language();
        assert_eq!(app.lang, Language::En);
        assert_eq!(app.db.get_setting("language")?, Some("en".to_string()));

        // Simulate mouse click on Japanese option in settings modal
        app.open_settings_modal();
        assert_eq!(app.modal, ActiveModal::Settings);
        app.hit_map.register(
            ratatui::layout::Rect::new(40, 10, 18, 1),
            HitTarget::ModalSelectLanguage(Language::Ja),
        );
        app.handle_mouse_click(45, 10)?;
        assert_eq!(app.lang, Language::Ja);
        assert_eq!(app.db.get_setting("language")?, Some("ja".to_string()));

        // Reopen app with same database: should restore Japanese
        let db2 = Database::new(Some(&db_path))?;
        let app2 = App::new(db2)?;
        assert_eq!(app2.lang, Language::Ja);

        let _ = std::fs::remove_file(&db_path);
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
