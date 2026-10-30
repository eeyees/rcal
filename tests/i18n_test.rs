#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rcal::i18n::Language;

    #[test]
    fn test_i18n_toggle_and_labels() {
        let lang_ko = Language::Ko;
        let lang_en = lang_ko.toggle();

        assert_eq!(lang_en, Language::En);
        assert_eq!(lang_ko.label(), "한국어");
        assert_eq!(lang_en.label(), "English");

        // View names
        assert_eq!(lang_ko.view_month(), "1:월간");
        assert_eq!(lang_en.view_month(), "1:Month");
        assert_eq!(lang_ko.view_week(), "2:주간");
        assert_eq!(lang_en.view_week(), "2:Week");

        // Action buttons
        assert_eq!(lang_ko.btn_new_event(), "+ 새 일정");
        assert_eq!(lang_en.btn_new_event(), "+ New");
        assert_eq!(lang_ko.btn_sync(), "⟳ 동기화");
        assert_eq!(lang_en.btn_sync(), "⟳ Sync");
    }

    #[test]
    fn test_i18n_date_formatting() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();

        let ko_month_year = Language::Ko.format_month_year(date);
        assert_eq!(ko_month_year, "2026년 10월");

        let en_month_year = Language::En.format_month_year(date);
        assert_eq!(en_month_year, "October 2026");

        let ko_headers = Language::Ko.weekday_headers();
        assert_eq!(ko_headers, ["일", "월", "화", "수", "목", "금", "토"]);

        let en_headers = Language::En.weekday_headers();
        assert_eq!(en_headers, ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]);
    }
}
