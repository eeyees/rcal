use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Translations {
    // Identity & Meta
    pub code: &'static str,
    pub label: &'static str,
    pub short_label: &'static str,

    // Views
    pub view_month: &'static str,
    pub view_week: &'static str,
    pub view_day: &'static str,
    pub view_agenda: &'static str,

    // Header & Common buttons
    pub btn_today: &'static str,
    pub btn_new_event: &'static str,
    pub btn_sync: &'static str,
    pub btn_syncing: &'static str,
    pub btn_accounts: &'static str,
    pub btn_settings: &'static str,
    pub btn_help: &'static str,
    pub btn_save: &'static str,
    pub btn_cancel: &'static str,
    pub btn_edit: &'static str,
    pub btn_delete: &'static str,
    pub btn_close: &'static str,

    // Weekday headers & names
    pub weekday_headers: [&'static str; 7],
    pub week_start_sun_headers: [&'static str; 7],
    pub week_start_mon_headers: [&'static str; 7],
    pub week_start_sat_headers: [&'static str; 7],
    pub week_start_labels: [&'static str; 3],

    // Date formatting functions
    pub format_month_year: fn(NaiveDate) -> String,
    pub format_date_long: fn(NaiveDate) -> String,

    // Sidebar
    pub sidebar_calendars: &'static str,
    pub sidebar_manage: &'static str,
    pub sidebar_hide: &'static str,
    pub sidebar_quick_nav: &'static str,
    pub sidebar_tips: &'static [&'static str],
    pub sidebar_keys: &'static str,
    pub sidebar_key_tips: &'static [&'static str],

    // Footer
    pub footer_keys_label: &'static str,
    pub footer_view: &'static str,
    pub footer_sidebar: &'static str,
    pub footer_new: &'static str,
    pub footer_sync: &'static str,
    pub footer_lang: &'static str,
    pub footer_accounts: &'static str,
    pub footer_help: &'static str,
    pub footer_quit: &'static str,

    // Event Modals
    pub modal_new_event: &'static str,
    pub modal_edit_event: &'static str,
    pub modal_event_details: &'static str,
    pub field_title: &'static str,
    pub field_calendar: &'static str,
    pub field_date: &'static str,
    pub field_start_time: &'static str,
    pub field_end_time: &'static str,
    pub field_all_day: &'static str,
    pub field_location: &'static str,
    pub field_description: &'static str,
    pub placeholder_title: &'static str,
    pub placeholder_location: &'static str,
    pub placeholder_description: &'static str,
    pub cycle_hint: &'static str,
    pub yes_label: &'static str,
    pub no_label: &'static str,
    pub no_location: &'static str,
    pub no_description: &'static str,

    // Calendar Modal
    pub modal_cal_create_title: &'static str,
    pub modal_cal_manage_title: &'static str,
    pub cal_name_label: &'static str,
    pub cal_name_placeholder: &'static str,
    pub cal_colors: [&'static str; 7],
    pub cal_list_header: &'static str,
    pub cal_status_visible: &'static str,
    pub cal_status_hidden: &'static str,
    pub cal_default_badge: &'static str,
    pub cal_btn_create: &'static str,
    pub cal_key_hint: &'static str,

    // Account Modal
    pub modal_account_add_title: &'static str,
    pub modal_account_manage_title: &'static str,
    pub account_list_header: &'static str,
    pub account_empty_msg: &'static str,
    pub account_never_synced: &'static str,
    pub account_btn_add: &'static str,
    pub account_btn_sync_now: &'static str,

    // Settings Modal
    pub modal_settings_title: &'static str,
    pub settings_week_start_title: &'static str,
    pub settings_week_start_desc: &'static str,
    pub settings_theme_title: &'static str,
    pub settings_theme_desc: &'static str,
    pub settings_lang_title: &'static str,
    pub settings_hint_shortcuts: &'static str,
    pub settings_hint_day_select: &'static str,
    pub settings_hint_cycle: &'static str,
    pub settings_hint_lang: &'static str,
    pub settings_btn_close: &'static str,

    // Help Modal
    pub modal_help_title: &'static str,
    pub help_mouse_title: &'static str,
    pub help_mouse_lines: &'static [&'static str],
    pub help_keyboard_title: &'static str,
    pub help_keyboard_lines: &'static [&'static str],

    // View-specific
    pub day_view_hint: &'static str,
    pub all_day_label: &'static str,
    pub all_day_short: &'static str,
    pub add_all_day_tip: &'static str,
    pub more_events: fn(usize) -> String,

    // Status messages
    pub msg_welcome: &'static str,
    pub msg_saved: &'static str,
    pub msg_deleted: &'static str,
    pub msg_jump_today: fn(&str) -> String,
    pub msg_lang_changed: fn(&str) -> String,
    pub msg_sidebar_toggle: fn(bool) -> &'static str,
    pub msg_week_start_changed: fn(&str) -> String,
    pub msg_account_username_empty: &'static str,
    pub msg_account_updated: fn(&str) -> String,
    pub msg_account_added: fn(&str) -> String,
    pub msg_account_removed: &'static str,
    pub msg_calendar_created: fn(&str) -> String,
    pub msg_calendar_deleted: &'static str,
    pub msg_calendar_deleted_name: fn(&str) -> String,
    pub msg_calendar_toggled: &'static str,
}
