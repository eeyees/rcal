use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub mod hangul;
pub mod translations;
pub mod ko;
pub mod en;
pub mod ja;

pub use hangul::hangul_to_qwerty;
pub use translations::Translations;
pub use ko::KO;
pub use en::EN;
pub use ja::JA;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Ko,
    En,
    Ja,
}

impl Language {
    /// List of all supported languages in order of toggle cycling
    pub const ALL: &'static [Language] = &[Language::Ko, Language::En, Language::Ja];

    pub fn detect() -> Self {
        if let Ok(val) = std::env::var("RCAL_LANG") {
            if let Some(l) = Self::from_code(&val) {
                return l;
            }
        }
        let lang_env = std::env::var("LANG").unwrap_or_default().to_lowercase();
        let lc_all = std::env::var("LC_ALL").unwrap_or_default().to_lowercase();
        let lc_messages = std::env::var("LC_MESSAGES").unwrap_or_default().to_lowercase();
        let env_str = format!("{} {} {}", lang_env, lc_all, lc_messages);

        if env_str.contains("ko") {
            Language::Ko
        } else if env_str.contains("ja") {
            Language::Ja
        } else {
            // Default to Korean (primary locale for rcal), toggleable via 'L' or Settings anytime
            Language::Ko
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "ko" | "ko_kr" | "ko-kr" | "korean" => Some(Language::Ko),
            "en" | "en_us" | "en-us" | "english" => Some(Language::En),
            "ja" | "ja_jp" | "ja-jp" | "japanese" => Some(Language::Ja),
            _ => None,
        }
    }

    pub fn toggle(&self) -> Self {
        let all = Self::ALL;
        let pos = all.iter().position(|l| l == self).unwrap_or(0);
        all[(pos + 1) % all.len()]
    }

    /// Access the full structured translation catalog
    pub fn translations(&self) -> &'static Translations {
        match self {
            Language::Ko => &ko::KO,
            Language::En => &en::EN,
            Language::Ja => &ja::JA,
        }
    }

    /// Short alias for translations()
    pub fn t(&self) -> &'static Translations {
        self.translations()
    }

    pub fn code(&self) -> &'static str {
        self.translations().code
    }

    pub fn label(&self) -> &'static str {
        self.translations().label
    }

    pub fn short_label(&self) -> &'static str {
        self.translations().short_label
    }

    // View names
    pub fn view_month(&self) -> &'static str {
        self.translations().view_month
    }

    pub fn view_week(&self) -> &'static str {
        self.translations().view_week
    }

    pub fn view_day(&self) -> &'static str {
        self.translations().view_day
    }

    pub fn view_agenda(&self) -> &'static str {
        self.translations().view_agenda
    }

    // Buttons
    pub fn btn_today(&self) -> &'static str {
        self.translations().btn_today
    }

    pub fn btn_new_event(&self) -> &'static str {
        self.translations().btn_new_event
    }

    pub fn btn_sync(&self) -> &'static str {
        self.translations().btn_sync
    }

    pub fn btn_syncing(&self) -> &'static str {
        self.translations().btn_syncing
    }

    pub fn btn_accounts(&self) -> &'static str {
        self.translations().btn_accounts
    }

    pub fn btn_settings(&self) -> &'static str {
        self.translations().btn_settings
    }

    pub fn btn_help(&self) -> &'static str {
        self.translations().btn_help
    }

    pub fn btn_save(&self) -> &'static str {
        self.translations().btn_save
    }

    pub fn btn_cancel(&self) -> &'static str {
        self.translations().btn_cancel
    }

    pub fn btn_edit(&self) -> &'static str {
        self.translations().btn_edit
    }

    pub fn btn_delete(&self) -> &'static str {
        self.translations().btn_delete
    }

    pub fn btn_close(&self) -> &'static str {
        self.translations().btn_close
    }

    // Weekday headers
    pub fn weekday_headers(&self) -> [&'static str; 7] {
        self.translations().weekday_headers
    }

    pub fn weekday_name_short(&self, dow_idx_from_sun: usize) -> &'static str {
        self.weekday_headers()[dow_idx_from_sun % 7]
    }

    pub fn weekday_headers_for(&self, start: crate::model::WeekStartDay) -> [&'static str; 7] {
        match start {
            crate::model::WeekStartDay::Sunday => self.translations().week_start_sun_headers,
            crate::model::WeekStartDay::Monday => self.translations().week_start_mon_headers,
            crate::model::WeekStartDay::Saturday => self.translations().week_start_sat_headers,
        }
    }

    pub fn week_start_label(&self, start: crate::model::WeekStartDay) -> &'static str {
        match start {
            crate::model::WeekStartDay::Sunday => self.translations().week_start_labels[0],
            crate::model::WeekStartDay::Monday => self.translations().week_start_labels[1],
            crate::model::WeekStartDay::Saturday => self.translations().week_start_labels[2],
        }
    }

    // Date formatting
    pub fn format_month_year(&self, date: NaiveDate) -> String {
        (self.translations().format_month_year)(date)
    }

    pub fn format_date_long(&self, date: NaiveDate) -> String {
        (self.translations().format_date_long)(date)
    }

    // Sidebar titles
    pub fn sidebar_calendars(&self) -> &'static str {
        self.translations().sidebar_calendars
    }

    pub fn sidebar_manage(&self) -> &'static str {
        self.translations().sidebar_manage
    }

    pub fn sidebar_hide(&self) -> &'static str {
        self.translations().sidebar_hide
    }

    pub fn sidebar_quick_nav(&self) -> &'static str {
        self.translations().sidebar_quick_nav
    }

    pub fn sidebar_tips(&self) -> Vec<&'static str> {
        self.translations().sidebar_tips.to_vec()
    }

    pub fn sidebar_keys(&self) -> &'static str {
        self.translations().sidebar_keys
    }

    pub fn sidebar_key_tips(&self) -> Vec<&'static str> {
        self.translations().sidebar_key_tips.to_vec()
    }

    // Modals
    pub fn modal_new_event(&self) -> &'static str {
        self.translations().modal_new_event
    }

    pub fn modal_edit_event(&self) -> &'static str {
        self.translations().modal_edit_event
    }

    pub fn modal_event_details(&self) -> &'static str {
        self.translations().modal_event_details
    }

    pub fn field_title(&self) -> &'static str {
        self.translations().field_title
    }

    pub fn field_calendar(&self) -> &'static str {
        self.translations().field_calendar
    }

    pub fn field_date(&self) -> &'static str {
        self.translations().field_date
    }

    pub fn field_start_time(&self) -> &'static str {
        self.translations().field_start_time
    }

    pub fn field_end_time(&self) -> &'static str {
        self.translations().field_end_time
    }

    pub fn field_all_day(&self) -> &'static str {
        self.translations().field_all_day
    }

    pub fn field_location(&self) -> &'static str {
        self.translations().field_location
    }

    pub fn field_description(&self) -> &'static str {
        self.translations().field_description
    }

    pub fn placeholder_title(&self) -> &'static str {
        self.translations().placeholder_title
    }

    pub fn placeholder_location(&self) -> &'static str {
        self.translations().placeholder_location
    }

    pub fn placeholder_description(&self) -> &'static str {
        self.translations().placeholder_description
    }

    // Status messages
    pub fn msg_welcome(&self) -> &'static str {
        self.translations().msg_welcome
    }

    pub fn msg_saved(&self) -> &'static str {
        self.translations().msg_saved
    }

    pub fn msg_deleted(&self) -> &'static str {
        self.translations().msg_deleted
    }

    pub fn msg_jump_today(&self, date_str: &str) -> String {
        (self.translations().msg_jump_today)(date_str)
    }

    pub fn msg_lang_changed(&self) -> String {
        (self.translations().msg_lang_changed)(self.label())
    }

    pub fn msg_sidebar_toggle(&self, shown: bool) -> &'static str {
        (self.translations().msg_sidebar_toggle)(shown)
    }

    pub fn msg_week_start_changed(&self, label: &str) -> String {
        (self.translations().msg_week_start_changed)(label)
    }

    pub fn msg_account_username_empty(&self) -> &'static str {
        self.translations().msg_account_username_empty
    }

    pub fn msg_account_updated(&self, name: &str) -> String {
        (self.translations().msg_account_updated)(name)
    }

    pub fn msg_account_added(&self, name: &str) -> String {
        (self.translations().msg_account_added)(name)
    }

    pub fn msg_account_removed(&self) -> &'static str {
        self.translations().msg_account_removed
    }

    pub fn msg_calendar_created(&self, name: &str) -> String {
        (self.translations().msg_calendar_created)(name)
    }

    pub fn msg_calendar_deleted(&self) -> &'static str {
        self.translations().msg_calendar_deleted
    }

    pub fn msg_calendar_deleted_name(&self, name: &str) -> String {
        (self.translations().msg_calendar_deleted_name)(name)
    }

    pub fn msg_calendar_toggled(&self) -> &'static str {
        self.translations().msg_calendar_toggled
    }

    pub fn more_events(&self, count: usize) -> String {
        (self.translations().more_events)(count)
    }
}
