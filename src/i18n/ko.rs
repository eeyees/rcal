use super::Translations;
use chrono::{Datelike, NaiveDate};

fn format_month_year_ko(date: NaiveDate) -> String {
    format!("{}년 {:02}월", date.year(), date.month())
}

fn format_date_long_ko(date: NaiveDate) -> String {
    let dow = match date.weekday().num_days_from_sunday() {
        0 => "일요일",
        1 => "월요일",
        2 => "화요일",
        3 => "수요일",
        4 => "목요일",
        5 => "금요일",
        _ => "토요일",
    };
    format!("{}년 {}월 {}일 {}", date.year(), date.month(), date.day(), dow)
}

fn more_events_ko(count: usize) -> String {
    format!(" +{}개 더보기", count)
}

fn msg_jump_today_ko(date_str: &str) -> String {
    format!("오늘로 이동했습니다 ({})", date_str)
}

fn msg_lang_changed_ko(name: &str) -> String {
    format!("언어가 '{}'(으)로 변경되었습니다.", name)
}

fn msg_sidebar_toggle_ko(shown: bool) -> &'static str {
    if shown { "사이드바 표시" } else { "사이드바 숨김" }
}

fn msg_week_start_changed_ko(label: &str) -> String {
    format!("주간 시작 요일이 '{}'로 설정되었습니다.", label)
}

fn msg_account_updated_ko(name: &str) -> String {
    format!("계정 '{}' 정보가 업데이트되었습니다.", name)
}

fn msg_account_added_ko(name: &str) -> String {
    format!("계정 '{}'이(가) 추가되었습니다.", name)
}

fn msg_calendar_created_ko(name: &str) -> String {
    format!("캘린더 '{}'이(가) 생성되었습니다.", name)
}

fn msg_calendar_deleted_name_ko(name: &str) -> String {
    format!("캘린더 '{}'이(가) 삭제되었습니다.", name)
}

pub static KO: Translations = Translations {
    code: "ko",
    label: "한국어",
    short_label: "한",

    view_month: "1:월간",
    view_week: "2:주간",
    view_day: "3:일간",
    view_agenda: "4:목록",

    btn_today: "오늘",
    btn_new_event: "+ 새 일정",
    btn_sync: "⟳  동기화",
    btn_syncing: "⟳  동기화중",
    btn_accounts: "☁ 계정연동",
    btn_settings: "⚙ 설정",
    btn_help: "? 도움말",
    btn_save: "[ 저장 ]",
    btn_cancel: "[ 취소 ]",
    btn_edit: "[ 수정 (e) ]",
    btn_delete: "[ 삭제 (d) ]",
    btn_close: "[ 닫기 (q) ]",

    weekday_headers: ["일", "월", "화", "수", "목", "금", "토"],
    week_start_sun_headers: ["일", "월", "화", "수", "목", "금", "토"],
    week_start_mon_headers: ["월", "화", "수", "목", "금", "토", "일"],
    week_start_sat_headers: ["토", "일", "월", "화", "수", "목", "금"],
    week_start_labels: ["일요일 시작 (Sunday)", "월요일 시작 (Monday)", "토요일 시작 (Saturday)"],

    format_month_year: format_month_year_ko,
    format_date_long: format_date_long_ko,

    sidebar_calendars: " 내 캘린더 ",
    sidebar_manage: " + 캘린더 관리/추가 (C) ",
    sidebar_hide: "[ 사이드바 숨김 (b) ]",
    sidebar_quick_nav: "빠른 탐색:",
    sidebar_tips: &[
        " • 탭 클릭: 뷰 전환",
        " • 셀 클릭: 일정 추가",
        " • 일정 클릭: 상세/수정",
        " • [x] 클릭: 표시 토글",
        " • T 키: 테마 선택",
        " • L 키: 언어 전환",
    ],
    sidebar_keys: "단축키:",
    sidebar_key_tips: &[
        " • b: 사이드바 토글",
        " • h/j/k/l: 탐색",
        " • c/n: 새 일정",
        " • r: 실시간 동기화",
        " • T: 테마 선택",
        " • A: 계정 연동 설정",
        " • t: 오늘로 점프",
        " • q: 닫기 / 종료",
    ],

    footer_keys_label: "단축키:",
    footer_view: "뷰",
    footer_sidebar: "사이드바",
    footer_new: "새 일정",
    footer_sync: "동기화",
    footer_lang: "언어",
    footer_accounts: "계정",
    footer_help: "도움말",
    footer_quit: "종료",

    modal_new_event: " 새 일정 등록 ",
    modal_edit_event: " 일정 수정 ",
    modal_event_details: " 일정 상세 정보 ",
    field_title: " 제목:        ",
    field_calendar: " 캘린더:      ",
    field_date: " 날짜:        ",
    field_start_time: " 시작 시간:   ",
    field_end_time: " 종료 시간:   ",
    field_all_day: " 하루 종일:   ",
    field_location: " 장소:        ",
    field_description: " 메모:        ",
    placeholder_title: "(일정 제목을 입력하세요...)",
    placeholder_location: "(선택 입력: 회의실, 주소 등)",
    placeholder_description: "(상세 메모를 입력하세요)",
    cycle_hint: "(스페이스바로 변경)",
    yes_label: "[x] 예",
    no_label: "[ ] 아니오",
    no_location: "장소 미지정",
    no_description: "메모 없음",

    modal_cal_create_title: " 새 캘린더 생성 (카테고리 / 분류) ",
    modal_cal_manage_title: " 캘린더 관리 & 카테고리 설정 ",
    cal_name_label: " 캘린더 이름: ",
    cal_name_placeholder: "(예: 스터디, 운동, 가족, 사이드 프로젝트)",
    cal_colors: ["파랑", "초록", "노랑", "빨강", "보라", "청록", "주황"],
    cal_list_header: "등록된 캘린더 목록 (일정 등록 시 선택하여 분류 가능):",
    cal_status_visible: "[표시중]",
    cal_status_hidden: "[숨김]",
    cal_default_badge: "(기본)",
    cal_btn_create: " [ + 새 캘린더 생성 ] ",

    modal_account_add_title: " 캘린더 계정 연동 (Google / MS 365 / Samsung / Naver / Apple) ",
    modal_account_manage_title: " 캘린더 계정 & 클라우드 실시간 동기화 ",
    account_list_header: " 📋 연동된 캘린더 계정 목록 (삭제하려면 [ 삭제 ] 버튼 클릭):",
    account_empty_msg: "  연동된 클라우드 계정이 없습니다. 아래의 '[ + 계정 추가 ]'를 클릭하여 Google, Microsoft, Samsung, Naver, Apple 캘린더를 연동하세요!",
    account_never_synced: "동기화 이력 없음",
    account_btn_add: " [ + 계정 추가 ] ",
    account_btn_sync_now: " [ ⟳ 동기화 실행 ] ",

    modal_settings_title: " ⚙ 환경 설정 & 테마 (Settings & Themes) ",
    settings_week_start_title: "▶ 1. 주간 시작 요일 (First Day of Week)",
    settings_week_start_desc: "  달력(월간) 및 주간 타임라인의 첫 번째 요일을 지정합니다:",
    settings_theme_title: "▶ 2. 색상 테마 (Color Themes)",
    settings_theme_desc: "  클릭하거나 단축키 [T] / [c]로 테마를 순환 변경할 수 있습니다:",
    settings_lang_title: "▶ 3. 기본 언어 (Language)",
    settings_hint_shortcuts: "단축키: ",
    settings_hint_day_select: " 요일선택  ",
    settings_hint_cycle: " 순환이동  ",
    settings_hint_lang: " 언어전환",
    settings_btn_close: "[ 닫기 (Esc / Enter) ]",

    modal_help_title: " 키보드 & 마우스 조작 단축키 안내 ",
    help_mouse_title: "🖱  마우스 조작:",
    help_mouse_lines: &[
        "  • 상단 뷰 탭 클릭       : 월간 / 주간 / 일간 / 목록 뷰 즉시 전환",
        "  • [<] / [>] 클릭        : 이전 / 다음 기간 이동 (마우스 휠 스크롤 가능)",
        "  • [오늘] 클릭           : 오늘 날짜로 즉시 이동",
        "  • 날짜 셀 클릭          : 해당 날짜 선택",
        "  • 날짜 셀 더블 클릭     : 해당 날짜의 일간 뷰로 즉시 이동",
        "  • 시간 슬롯 클릭        : 해당 시간대에 새 일정 등록 모달 열기",
        "  • 일정 블록 클릭        : 일정 상세 보기 (수정 / 삭제 모달)",
        "  • [x] 체크박스 클릭     : 캘린더 표시 / 숨김 토글",
    ],
    help_keyboard_title: "⌨  키보드 단축키:",
    help_keyboard_lines: &[
        "  • 1 / 2 / 3 / 4         : 월간(1) / 주간(2) / 일간(3) / 목록(4) 전환",
        "  • b                     : 좌측 사이드바 표시 / 숨김 토글 (Ctrl+B)",
        "  • h / j / k / l         : 좌 / 하 / 상 / 우 날짜 및 시간대 이동",
        "  • [ / ]                 : 이전 / 다음 월/주/일 이동",
        "  • t                     : 오늘 날짜로 이동",
        "  • c 또는 n              : 새 일정 생성",
        "  • r                     : 클라우드 실시간 동기화 실행 (CalDAV/OAuth2)",
        "  • T                     : 테마 선택 모달 열기 (37종)",
        "  • A                     : 계정 관리 (Google / MS / Naver / Apple)",
        "  • C                     : 캘린더 카테고리 관리 및 추가/삭제",
        "  • o 또는 S              : 환경 설정 (주간 시작 요일 등)",
        "  • L                     : 언어 즉시 전환",
        "  • ?                     : 도움말 화면 표시",
        "  • q / Esc               : 닫기 또는 프로그램 종료",
    ],

    day_view_hint: "(시간 슬롯을 클릭하여 일정을 추가하세요)",
    all_day_label: "종일",
    all_day_short: "종일",
    add_all_day_tip: " + 클릭하여 종일 일정 추가...",
    more_events: more_events_ko,

    msg_welcome: "rcal에 오신 것을 환영합니다! '?'를 누르거나 [ ? 도움말 ]을 클릭하세요.",
    msg_saved: "일정이 성공적으로 저장되었습니다.",
    msg_deleted: "일정이 삭제되었습니다.",
    msg_jump_today: msg_jump_today_ko,
    msg_lang_changed: msg_lang_changed_ko,
    msg_sidebar_toggle: msg_sidebar_toggle_ko,
    msg_week_start_changed: msg_week_start_changed_ko,
    msg_account_username_empty: "계정 ID(사용자명)를 입력하세요.",
    msg_account_updated: msg_account_updated_ko,
    msg_account_added: msg_account_added_ko,
    msg_account_removed: "계정이 삭제되었습니다.",
    msg_calendar_created: msg_calendar_created_ko,
    msg_calendar_deleted: "캘린더가 삭제되었습니다.",
    msg_calendar_deleted_name: msg_calendar_deleted_name_ko,
    msg_calendar_toggled: "캘린더 표시 상태가 변경되었습니다.",
};
