use super::Translations;
use chrono::NaiveDate;

fn format_month_year_en(date: NaiveDate) -> String {
    date.format("%B %Y").to_string()
}

fn format_date_long_en(date: NaiveDate) -> String {
    date.format("%A, %B %d, %Y").to_string()
}

fn more_events_en(count: usize) -> String {
    format!(" +{} more", count)
}

fn msg_jump_today_en(date_str: &str) -> String {
    format!("Jumped to today ({})", date_str)
}

fn msg_lang_changed_en(name: &str) -> String {
    format!("Language changed to '{}'.", name)
}

fn msg_sidebar_toggle_en(shown: bool) -> &'static str {
    if shown { "Sidebar shown" } else { "Sidebar hidden" }
}

fn msg_week_start_changed_en(label: &str) -> String {
    format!("Week start day set to '{}'.", label)
}

fn msg_account_updated_en(name: &str) -> String {
    format!("Account '{}' updated.", name)
}

fn msg_account_added_en(name: &str) -> String {
    format!("Added account '{}'", name)
}

fn msg_calendar_created_en(name: &str) -> String {
    format!("Created calendar '{}'", name)
}

fn msg_calendar_deleted_name_en(name: &str) -> String {
    format!("Deleted calendar '{}'", name)
}

pub static EN: Translations = Translations {
    code: "en",
    label: "English",
    short_label: "EN",

    view_month: "1:Month",
    view_week: "2:Week",
    view_day: "3:Day",
    view_agenda: "4:Agenda",

    btn_today: "Today",
    btn_new_event: "+ New",
    btn_sync: "⟳  Sync",
    btn_syncing: "⟳  Syncing",
    btn_accounts: "☁ Accounts",
    btn_settings: "⚙ Setup",
    btn_help: "? Help",
    btn_save: "[ Save ]",
    btn_cancel: "[ Cancel ]",
    btn_edit: "[ Edit (e) ]",
    btn_delete: "[ Delete (d) ]",
    btn_close: "[ Close (q) ]",

    weekday_headers: ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"],
    week_start_sun_headers: ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"],
    week_start_mon_headers: ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"],
    week_start_sat_headers: ["SAT", "SUN", "MON", "TUE", "WED", "THU", "FRI"],
    week_start_labels: ["Sunday start", "Monday start", "Saturday start"],

    format_month_year: format_month_year_en,
    format_date_long: format_date_long_en,

    sidebar_calendars: " My Calendars ",
    sidebar_manage: " + Manage Calendars (C) ",
    sidebar_hide: "[ Hide (b) ]",
    sidebar_quick_nav: "Quick Navigation:",
    sidebar_tips: &[
        " • Click tab: switch view",
        " • Click slot: add event",
        " • Click event: details/edit",
        " • [x] Checkbox: toggle cal",
        " • 'L' key: toggle language",
    ],
    sidebar_keys: "Keys:",
    sidebar_key_tips: &[
        " • b: Toggle sidebar",
        " • h/j/k/l: Navigate",
        " • c/n: New event",
        " • r: Sync now",
        " • A: Accounts modal",
        " • t: Jump to today",
        " • q: Close / Quit",
    ],

    footer_keys_label: "Keys:",
    footer_view: "View",
    footer_sidebar: "Sidebar",
    footer_new: "New",
    footer_sync: "Sync",
    footer_lang: "Lang",
    footer_accounts: "Accounts",
    footer_help: "Help",
    footer_quit: "Quit",

    modal_new_event: " New Event ",
    modal_edit_event: " Edit Event ",
    modal_event_details: " Event Details ",
    field_title: " Title:       ",
    field_calendar: " Calendar:    ",
    field_date: " Date:        ",
    field_start_time: " Start Time:  ",
    field_end_time: " End Time:    ",
    field_all_day: " All Day:     ",
    field_location: " Location:    ",
    field_description: " Description: ",
    placeholder_title: "(Type event title...)",
    placeholder_location: "(Optional location)",
    placeholder_description: "(Optional notes)",
    cycle_hint: "(Space to cycle)",
    yes_label: "[x] Yes",
    no_label: "[ ] No",
    no_location: "No location specified",
    no_description: "No description",

    modal_cal_create_title: " Create New Calendar ",
    modal_cal_manage_title: " Manage Calendars & Categories ",
    cal_name_label: " Name:          ",
    cal_name_placeholder: "(e.g. Study, Fitness, Family, Side Project)",
    cal_colors: ["Blue", "Green", "Yellow", "Red", "Purple", "Cyan", "Orange"],
    cal_list_header: "Registered Calendars (Select when creating events):",
    cal_status_visible: "[Visible]",
    cal_status_hidden: "[Hidden]",
    cal_default_badge: "(Default)",
    cal_btn_create: " [ + Create Calendar ] ",

    modal_account_add_title: " Add Calendar Account (Google / MS 365 / Samsung / Naver / Apple) ",
    modal_account_manage_title: " Calendar Accounts & Multi-Cloud Sync ",
    account_list_header: " 📋 Configured Sync Accounts (Click [ Delete ] to remove):",
    account_empty_msg: "  No cloud accounts linked yet. Click '[ + Add Account ]' below to link Google, Microsoft, Samsung, Naver, or Apple calendars!",
    account_never_synced: "Never",
    account_btn_add: " [ + Add Account ] ",
    account_btn_sync_now: " [ ⟳ Sync Now ] ",

    modal_settings_title: " ⚙ Settings & Themes ",
    settings_week_start_title: "▶ 1. First Day of Week",
    settings_week_start_desc: "  Choose which day starts the week in Month and Week views:",
    settings_theme_title: "▶ 2. Color Themes",
    settings_theme_desc: "  Click any theme or press [T] / [c] to cycle color themes:",
    settings_lang_title: "▶ 3. Display Language",
    settings_hint_shortcuts: "Shortcuts: ",
    settings_hint_day_select: " Select Day  ",
    settings_hint_cycle: " Cycle  ",
    settings_hint_lang: " Language",
    settings_btn_close: "[ Close (Esc / Enter) ]",

    modal_help_title: " Keyboard & Mouse Shortcuts ",
    help_mouse_title: "🖱  Mouse Controls:",
    help_mouse_lines: &[
        "  • Click View Tabs       : Switch between Month, Week, Day, Agenda",
        "  • Click [<] / [>]       : Navigate backward / forward in time (or wheel scroll)",
        "  • Click [Today]         : Jump straight to today's date",
        "  • Click Day Cell        : Select date in Month view",
        "  • Double Click Day      : Open Day view for that date",
        "  • Click Time Slot       : Create new event at that hour",
        "  • Click Event Block     : View event details & edit/delete",
        "  • Click [x] Checkbox    : Toggle calendar visibility ON/OFF",
    ],
    help_keyboard_title: "⌨  Keyboard Shortcuts:",
    help_keyboard_lines: &[
        "  • 1 / 2 / 3 / 4         : Month / Week / Day / Agenda view",
        "  • b                     : Toggle left sidebar visibility (Ctrl+B)",
        "  • h / j / k / l         : Navigate days / weeks / hours",
        "  • [ / ]                 : Previous / Next period",
        "  • t                     : Jump to today",
        "  • c / n                 : Create new event",
        "  • r                     : Cloud sync now (CalDAV/OAuth2)",
        "  • A                     : Accounts modal (Google / MS / Naver / Apple)",
        "  • C                     : Manage & Create calendars",
        "  • o / S                 : Settings modal (First day of week)",
        "  • L                     : Toggle display language",
        "  • ?                     : Show help screen",
        "  • q / Esc               : Close modal / Quit rcal",
    ],

    day_view_hint: "(Click any time slot to add an event)",
    all_day_label: "All-day",
    all_day_short: "All",
    add_all_day_tip: " + Click to add all-day event...",
    more_events: more_events_en,

    msg_welcome: "Welcome to rcal! Press ? or click [ Help ] for shortcuts.",
    msg_saved: "Event saved successfully.",
    msg_deleted: "Event deleted.",
    msg_jump_today: msg_jump_today_en,
    msg_lang_changed: msg_lang_changed_en,
    msg_sidebar_toggle: msg_sidebar_toggle_en,
    msg_week_start_changed: msg_week_start_changed_en,
    msg_account_username_empty: "Username cannot be empty.",
    msg_account_updated: msg_account_updated_en,
    msg_account_added: msg_account_added_en,
    msg_account_removed: "Account removed.",
    msg_calendar_created: msg_calendar_created_en,
    msg_calendar_deleted: "Calendar deleted.",
    msg_calendar_deleted_name: msg_calendar_deleted_name_en,
    msg_calendar_toggled: "Calendar visibility toggled",
};
