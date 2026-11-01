use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub mod hangul;
pub mod translations;
pub mod ko;
pub mod en;
pub mod ja;
pub mod locales;

pub use hangul::hangul_to_qwerty;
pub use translations::Translations;
pub use ko::KO;
pub use en::EN;
pub use ja::JA;
pub use locales::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Ko,
    En,
    Ja,
    ZhCn,
    ZhTw,
    Mn,
    Es,
    Fr,
    De,
    Ru,
    Pt,
    It,
    Vi,
    Id,
    Th,
    Ar,
    Hi,
    Tr,
    Nl,
    Pl,
    Sv,
}

impl Language {
    /// List of all supported languages in order of toggle cycling
    pub const ALL: &'static [Language] = &[
        Language::Ko,
        Language::En,
        Language::Ja,
        Language::ZhCn,
        Language::ZhTw,
        Language::Mn,
        Language::Es,
        Language::Fr,
        Language::De,
        Language::Ru,
        Language::Pt,
        Language::It,
        Language::Vi,
        Language::Id,
        Language::Th,
        Language::Ar,
        Language::Hi,
        Language::Tr,
        Language::Nl,
        Language::Pl,
        Language::Sv,
    ];

    /// Detects the system language.
    /// If no language is explicitly configured, follows the OS system language setting.
    /// If the system language is unsupported or cannot be determined, defaults to English (Language::En).
    pub fn detect() -> Self {
        // 1. Explicit override via RCAL_LANG environment variable
        if let Ok(val) = std::env::var("RCAL_LANG") {
            if let Some(l) = Self::from_code(&val) {
                return l;
            }
        }

        // 2. Explicit override via LC_ALL (standard POSIX highest priority locale override)
        if let Ok(val) = std::env::var("LC_ALL") {
            if !val.trim().is_empty() {
                if let Some(l) = Self::from_code(&val) {
                    return l;
                }
            }
        }

        // 3. System locale detection across OS platforms (macOS CFLocale, Windows API, Linux env)
        let locales: Vec<String> = sys_locale::get_locales().collect();
        if !locales.is_empty() {
            for loc in &locales {
                if let Some(l) = Self::from_code(loc) {
                    return l;
                }
            }
            // System language was detected from the OS, but is not in our supported list: default to English
            return Language::En;
        }

        // 4. POSIX environment variables fallback (LC_MESSAGES, LANG, LANGUAGE)
        for var in ["LC_MESSAGES", "LANG", "LANGUAGE"] {
            if let Ok(val) = std::env::var(var) {
                if !val.trim().is_empty() {
                    for part in val.split(':') {
                        if let Some(l) = Self::from_code(part) {
                            return l;
                        }
                    }
                }
            }
        }

        // 5. Default fallback to English if unsupported or undetermined
        Language::En
    }

    /// Helper to detect supported language from an iterator of locale identifier strings.
    /// Returns the first supported language, or defaults to English if none are supported.
    pub fn detect_from_locales<I, S>(locales: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for loc in locales {
            let s = loc.as_ref();
            if let Some(l) = Self::from_code(s) {
                return l;
            }
        }
        Language::En
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let clean = code.trim();
        if clean.is_empty() {
            return None;
        }

        // Strip character encoding (.UTF-8) and POSIX modifiers (@euro)
        let base: String = clean
            .chars()
            .take_while(|&c| c != '.' && c != '@')
            .collect();
        let lower = base.to_lowercase().replace('_', "-");
        let s = lower.trim();

        // 1. Direct match on code, BCP 47 tags, or language names
        match s {
            "ko" | "ko-kr" | "ko-kp" | "korean" => return Some(Language::Ko),
            "en" | "en-us" | "en-gb" | "en-ca" | "en-au" | "en-nz" | "en-ie" | "en-za" | "english" => return Some(Language::En),
            "ja" | "ja-jp" | "japanese" => return Some(Language::Ja),
            "zh-cn" | "zh-sg" | "zh-hans" | "zh-hans-cn" | "zh-hans-sg" | "zh" | "chinese" | "chinese-simplified" => return Some(Language::ZhCn),
            "zh-tw" | "zh-hk" | "zh-mo" | "zh-hant" | "zh-hant-tw" | "zh-hant-hk" | "zh-hant-mo" | "chinese-traditional" => return Some(Language::ZhTw),
            "mn" | "mn-mn" | "mongolian" => return Some(Language::Mn),
            "es" | "es-es" | "es-mx" | "es-419" | "es-ar" | "spanish" => return Some(Language::Es),
            "fr" | "fr-fr" | "fr-ca" | "fr-be" | "fr-ch" | "french" => return Some(Language::Fr),
            "de" | "de-de" | "de-at" | "de-ch" | "german" => return Some(Language::De),
            "ru" | "ru-ru" | "ru-by" | "ru-kz" | "russian" => return Some(Language::Ru),
            "pt" | "pt-br" | "pt-pt" | "portuguese" => return Some(Language::Pt),
            "it" | "it-it" | "it-ch" | "italian" => return Some(Language::It),
            "vi" | "vi-vn" | "vietnamese" => return Some(Language::Vi),
            "id" | "id-id" | "indonesian" => return Some(Language::Id),
            "th" | "th-th" | "thai" => return Some(Language::Th),
            "ar" | "ar-sa" | "ar-ae" | "ar-eg" | "arabic" => return Some(Language::Ar),
            "hi" | "hi-in" | "hindi" => return Some(Language::Hi),
            "tr" | "tr-tr" | "turkish" => return Some(Language::Tr),
            "nl" | "nl-nl" | "nl-be" | "dutch" => return Some(Language::Nl),
            "pl" | "pl-pl" | "polish" => return Some(Language::Pl),
            "sv" | "sv-se" | "sv-fi" | "swedish" => return Some(Language::Sv),
            _ => {}
        }

        // 2. Chinese variants matching
        if s.starts_with("zh-hant") || s.starts_with("zh-tw") || s.starts_with("zh-hk") || s.starts_with("zh-mo") {
            return Some(Language::ZhTw);
        }
        if s.starts_with("zh-hans") || s.starts_with("zh-cn") || s.starts_with("zh-sg") || s == "zh" {
            return Some(Language::ZhCn);
        }

        // 3. Primary language subtag prefix matching (e.g. "en-PH" -> "en", "es-CO" -> "es")
        if let Some(prefix) = s.split('-').next() {
            if prefix != s {
                match prefix {
                    "ko" => return Some(Language::Ko),
                    "en" => return Some(Language::En),
                    "ja" => return Some(Language::Ja),
                    "mn" => return Some(Language::Mn),
                    "es" => return Some(Language::Es),
                    "fr" => return Some(Language::Fr),
                    "de" => return Some(Language::De),
                    "ru" => return Some(Language::Ru),
                    "pt" => return Some(Language::Pt),
                    "it" => return Some(Language::It),
                    "vi" => return Some(Language::Vi),
                    "id" => return Some(Language::Id),
                    "th" => return Some(Language::Th),
                    "ar" => return Some(Language::Ar),
                    "hi" => return Some(Language::Hi),
                    "tr" => return Some(Language::Tr),
                    "nl" => return Some(Language::Nl),
                    "pl" => return Some(Language::Pl),
                    "sv" => return Some(Language::Sv),
                    _ => {}
                }
            }
        }

        None
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
            Language::ZhCn => &locales::ZH_CN,
            Language::ZhTw => &locales::ZH_TW,
            Language::Mn => &locales::MN,
            Language::Es => &locales::ES,
            Language::Fr => &locales::FR,
            Language::De => &locales::DE,
            Language::Ru => &locales::RU,
            Language::Pt => &locales::PT,
            Language::It => &locales::IT,
            Language::Vi => &locales::VI,
            Language::Id => &locales::ID,
            Language::Th => &locales::TH,
            Language::Ar => &locales::AR,
            Language::Hi => &locales::HI,
            Language::Tr => &locales::TR,
            Language::Nl => &locales::NL,
            Language::Pl => &locales::PL,
            Language::Sv => &locales::SV,
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

    pub fn btn_theme(&self) -> &'static str {
        match self {
            Language::Ko => "🎨  테마",
            Language::Ja => "🎨  テーマ",
            Language::ZhCn => "🎨  主题",
            Language::ZhTw => "🎨  主題",
            _ => "🎨  Theme",
        }
    }

    pub fn modal_theme_title(&self) -> &'static str {
        match self {
            Language::Ko => " 🎨  컬러 테마 선택 (Select Theme) ",
            Language::Ja => " 🎨  テーマの選択 (Select Theme) ",
            Language::ZhCn => " 🎨  主题选择 (Select Theme) ",
            Language::ZhTw => " 🎨  主題選擇 (Select Theme) ",
            _ => " 🎨  Select Color Theme ",
        }
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
