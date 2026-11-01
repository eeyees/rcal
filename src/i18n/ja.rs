use super::Translations;
use chrono::{Datelike, NaiveDate};

fn format_month_year_ja(date: NaiveDate) -> String {
    format!("{}年 {:02}月", date.year(), date.month())
}

fn format_date_long_ja(date: NaiveDate) -> String {
    let dow = match date.weekday().num_days_from_sunday() {
        0 => "日曜日",
        1 => "月曜日",
        2 => "火曜日",
        3 => "水曜日",
        4 => "木曜日",
        5 => "金曜日",
        _ => "土曜日",
    };
    format!("{}年 {}月 {}日 {}", date.year(), date.month(), date.day(), dow)
}

fn more_events_ja(count: usize) -> String {
    format!(" +{}件 他", count)
}

fn msg_jump_today_ja(date_str: &str) -> String {
    format!("今日へ移動しました ({})", date_str)
}

fn msg_lang_changed_ja(name: &str) -> String {
    format!("言語を '{}' に変更しました。", name)
}

fn msg_sidebar_toggle_ja(shown: bool) -> &'static str {
    if shown { "サイドバー表示" } else { "サイドバー非表示" }
}

fn msg_week_start_changed_ja(label: &str) -> String {
    format!("週の開始曜日が '{}' に設定されました。", label)
}

fn msg_account_updated_ja(name: &str) -> String {
    format!("アカウント '{}' の情報を更新しました。", name)
}

fn msg_account_added_ja(name: &str) -> String {
    format!("アカウント '{}' を追加しました。", name)
}

fn msg_calendar_created_ja(name: &str) -> String {
    format!("カレンダー '{}' を作成しました。", name)
}

fn msg_calendar_deleted_name_ja(name: &str) -> String {
    format!("カレンダー '{}' を削除しました。", name)
}

pub static JA: Translations = Translations {
    code: "ja",
    label: "日本語",
    short_label: "日",

    view_month: "1:月間",
    view_week: "2:週間",
    view_day: "3:日間",
    view_agenda: "4:予定表",

    btn_today: "今日",
    btn_new_event: "+ 新規予定",
    btn_sync: "⟳ 同期",
    btn_syncing: "⟳ 同期中",
    btn_accounts: "アカウント",
    btn_settings: "⚙ 設定",
    btn_help: "? ヘルプ",
    btn_save: "[ 保存 ]",
    btn_cancel: "[ キャンセル ]",
    btn_edit: "[ 編集 (e) ]",
    btn_delete: "[ 削除 (d) ]",
    btn_close: "[ 閉じる (q) ]",

    weekday_headers: ["日", "月", "火", "水", "木", "金", "土"],
    week_start_sun_headers: ["日", "月", "火", "水", "木", "金", "土"],
    week_start_mon_headers: ["月", "火", "水", "木", "金", "土", "日"],
    week_start_sat_headers: ["土", "日", "月", "火", "水", "木", "金"],
    week_start_labels: ["日曜日始まり (Sunday)", "月曜日始まり (Monday)", "土曜日始まり (Saturday)"],

    format_month_year: format_month_year_ja,
    format_date_long: format_date_long_ja,

    sidebar_calendars: " マイカレンダー ",
    sidebar_manage: " + カレンダー管理/追加 (C) ",
    sidebar_hide: "[ サイドバー非表示 (b) ]",
    sidebar_quick_nav: "クイックナビ:",
    sidebar_tips: &[
        " • タブクリック: ビュー切替",
        " • セルクリック: 予定追加",
        " • 予定クリック: 詳細/編集",
        " • [x] チェック: 表示切替",
        " • Lキー: 言語切替",
    ],
    sidebar_keys: "ショートカット:",
    sidebar_key_tips: &[
        " • b: サイドバー表示切替",
        " • h/j/k/l: 移動・選択",
        " • c/n: 新規予定",
        " • r: リアルタイム同期",
        " • A: アカウント連携設定",
        " • t: 今日に移動",
        " • q: 閉じる / 終了",
    ],

    footer_keys_label: "操作:",
    footer_view: "ビュー",
    footer_sidebar: "サイドバー",
    footer_new: "新規",
    footer_sync: "同期",
    footer_lang: "言語",
    footer_accounts: "アカウント",
    footer_help: "ヘルプ",
    footer_quit: "終了",

    modal_new_event: " 新規予定登録 ",
    modal_edit_event: " 予定の編集 ",
    modal_event_details: " 予定の詳細情報 ",
    field_title: " タイトル:    ",
    field_calendar: " カレンダー:  ",
    field_date: " 日付:        ",
    field_start_time: " 開始時刻:    ",
    field_end_time: " 終了時刻:    ",
    field_all_day: " 終日:        ",
    field_location: " 場所:        ",
    field_description: " メモ:        ",
    placeholder_title: "(予定タイトルを入力...)",
    placeholder_location: "(任意: 会議室・場所など)",
    placeholder_description: "(詳細メモを入力)",
    cycle_hint: "(スペースキーで変更)",
    yes_label: "[x] はい",
    no_label: "[ ] いいえ",
    no_location: "場所未指定",
    no_description: "メモなし",

    modal_cal_create_title: " 新規カレンダー作成 (カテゴリ設定) ",
    modal_cal_manage_title: " カレンダー管理 & カテゴリ設定 ",
    cal_name_label: " カレンダー名: ",
    cal_name_placeholder: "(例: 勉強, フィットネス, 家族, プロジェクト)",
    cal_colors: ["青", "緑", "黄", "赤", "紫", "シアン", "オレンジ"],
    cal_list_header: "登録済みカレンダー一覧 (予定作成時に選択可能):",
    cal_status_visible: "[表示中]",
    cal_status_hidden: "[非表示]",
    cal_default_badge: "(デフォルト)",
    cal_btn_create: " [ + 新規カレンダー作成 ] ",

    modal_account_add_title: " カレンダーアカウント連携 (Google / MS 365 / Samsung / Naver / Apple) ",
    modal_account_manage_title: " カレンダーアカウント & クラウド同期 ",
    account_list_header: " 📋 連携済みカレンダーアカウント一覧 (削除は [ 削除 ] をクリック):",
    account_empty_msg: "  連携中のクラウドアカウントがありません。下の '[ + アカウント追加 ]' から連携してください！",
    account_never_synced: "同期履歴なし",
    account_btn_add: " [ + アカウント追加 ] ",
    account_btn_sync_now: " [ ⟳ 今すぐ同期 ] ",

    modal_settings_title: " ⚙ 環境設定＆テーマ (Settings & Themes) ",
    settings_week_start_title: "▶ 1. 週の開始曜日 (First Day of Week)",
    settings_week_start_desc: "  カレンダー(月間)および週間タイムラインの最初の曜日を指定します:",
    settings_theme_title: "▶ 2. カラーテーマ (Color Themes)",
    settings_theme_desc: "  クリックまたはショートカット [T] / [c] でテーマを切替:",
    settings_lang_title: "▶ 3. 表示言語 (Language)",
    settings_hint_shortcuts: "操作案内: ",
    settings_hint_day_select: " 曜日選択  ",
    settings_hint_cycle: " 巡回切替  ",
    settings_hint_lang: " 言語切替",
    settings_btn_close: "[ 閉じる (Esc / Enter) ]",

    modal_help_title: " キーボード＆マウス操作ショートカット案内 ",
    help_mouse_title: "🖱  マウス操作:",
    help_mouse_lines: &[
        "  • 画面上部タブクリック   : 月間 / 週間 / 日間 / 予定表ビュー即時切替",
        "  • [<] / [>] クリック     : 前 / 次の期間へ移動 (マウスホイール操作可)",
        "  • [今日] クリック        : 今日の日付へ即時移動",
        "  • 日付セルクリック       : その日付を選択",
        "  • 日付セルダブルクリック : その日付の日間ビューへ即時移動",
        "  • 時間枠クリック         : その時間帯に新規予定作成モーダルを開く",
        "  • 予定ブロッククリック   : 予定の詳細表示 (編集 / 削除モーダル)",
        "  • [x] チェッククリック   : カレンダー表示 / 非表示トグル",
    ],
    help_keyboard_title: "⌨  キーボードショートカット:",
    help_keyboard_lines: &[
        "  • 1 / 2 / 3 / 4         : 月間(1) / 週間(2) / 日間(3) / 予定表(4) 切替",
        "  • b                     : 左サイドバー表示 / 非表示切替 (Ctrl+B)",
        "  • h / j / k / l         : 左 / 下 / 上 / 右 日付・時間移動",
        "  • [ / ]                 : 前 / 次の月・週・日へ移動",
        "  • t                     : 今日の日付へ移動",
        "  • c または n            : 新規予定作成",
        "  • r                     : クラウド同期実行 (CalDAV/OAuth2)",
        "  • A                     : アカウント管理 (Google / MS / Naver / Apple)",
        "  • C                     : カレンダーカテゴリ管理・追加・削除",
        "  • o または S            : 環境設定 (週の開始曜日など)",
        "  • L                     : 言語即時切替 (日 ↔ 韓 ↔ 英)",
        "  • ?                     : ヘルプ画面表示",
        "  • q / Esc               : 閉じる / プログラム終了",
    ],

    day_view_hint: "(時間枠をクリックして予定を追加)",
    all_day_label: "終日",
    all_day_short: "終日",
    add_all_day_tip: " + クリックして終日予定を追加...",
    more_events: more_events_ja,

    msg_welcome: "rcalへようこそ！ '?' キーまたは [ ? ヘルプ ] をクリックしてください。",
    msg_saved: "予定が正常に保存されました。",
    msg_deleted: "予定が削除されました。",
    msg_jump_today: msg_jump_today_ja,
    msg_lang_changed: msg_lang_changed_ja,
    msg_sidebar_toggle: msg_sidebar_toggle_ja,
    msg_week_start_changed: msg_week_start_changed_ja,
    msg_account_username_empty: "アカウントID(ユーザー名)を入力してください。",
    msg_account_updated: msg_account_updated_ja,
    msg_account_added: msg_account_added_ja,
    msg_account_removed: "アカウントが削除されました。",
    msg_calendar_created: msg_calendar_created_ja,
    msg_calendar_deleted: "カレンダーが削除されました。",
    msg_calendar_deleted_name: msg_calendar_deleted_name_ja,
    msg_calendar_toggled: "カレンダーの表示状態を変更しました。",
};
