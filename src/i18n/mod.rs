use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Ko,
    En,
}

impl Language {
    pub fn detect() -> Self {
        let lang_env = std::env::var("LANG").unwrap_or_default().to_lowercase();
        let lc_all = std::env::var("LC_ALL").unwrap_or_default().to_lowercase();
        let lc_messages = std::env::var("LC_MESSAGES").unwrap_or_default().to_lowercase();

        if lang_env.contains("ko") || lc_all.contains("ko") || lc_messages.contains("ko") {
            Language::Ko
        } else {
            // Default to Korean since user conversation is in Korean,
            // while allowing 1-key toggle with 'L' anytime
            Language::Ko
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Ko => Language::En,
            Language::En => Language::Ko,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::Ko => "한국어",
            Language::En => "English",
        }
    }

    // View names
    pub fn view_month(&self) -> &'static str {
        match self {
            Language::Ko => "1:월간",
            Language::En => "1:Month",
        }
    }

    pub fn view_week(&self) -> &'static str {
        match self {
            Language::Ko => "2:주간",
            Language::En => "2:Week",
        }
    }

    pub fn view_day(&self) -> &'static str {
        match self {
            Language::Ko => "3:일간",
            Language::En => "3:Day",
        }
    }

    pub fn view_agenda(&self) -> &'static str {
        match self {
            Language::Ko => "4:목록",
            Language::En => "4:Agenda",
        }
    }

    // Buttons
    pub fn btn_today(&self) -> &'static str {
        match self {
            Language::Ko => "오늘",
            Language::En => "Today",
        }
    }

    pub fn btn_new_event(&self) -> &'static str {
        match self {
            Language::Ko => "+ 새 일정",
            Language::En => "+ New",
        }
    }

    pub fn btn_sync(&self) -> &'static str {
        match self {
            Language::Ko => "⟳ 동기화",
            Language::En => "⟳ Sync",
        }
    }

    pub fn btn_syncing(&self) -> &'static str {
        match self {
            Language::Ko => "⟳ 동기화중",
            Language::En => "⟳ Syncing",
        }
    }

    pub fn btn_accounts(&self) -> &'static str {
        match self {
            Language::Ko => "계정연동",
            Language::En => "Accounts",
        }
    }

    pub fn btn_help(&self) -> &'static str {
        match self {
            Language::Ko => "? 도움말",
            Language::En => "? Help",
        }
    }

    pub fn btn_save(&self) -> &'static str {
        match self {
            Language::Ko => "[ 저장 ]",
            Language::En => "[ Save ]",
        }
    }

    pub fn btn_cancel(&self) -> &'static str {
        match self {
            Language::Ko => "[ 취소 ]",
            Language::En => "[ Cancel ]",
        }
    }

    pub fn btn_edit(&self) -> &'static str {
        match self {
            Language::Ko => "[ 수정 (e) ]",
            Language::En => "[ Edit (e) ]",
        }
    }

    pub fn btn_delete(&self) -> &'static str {
        match self {
            Language::Ko => "[ 삭제 (d) ]",
            Language::En => "[ Delete (d) ]",
        }
    }

    pub fn btn_close(&self) -> &'static str {
        match self {
            Language::Ko => "[ 닫기 (q) ]",
            Language::En => "[ Close (q) ]",
        }
    }

    // Weekday headers
    pub fn weekday_headers(&self) -> [&'static str; 7] {
        match self {
            Language::Ko => ["일", "월", "화", "수", "목", "금", "토"],
            Language::En => ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"],
        }
    }

    pub fn weekday_name_short(&self, dow_idx_from_sun: usize) -> &'static str {
        self.weekday_headers()[dow_idx_from_sun % 7]
    }

    // Date formatting
    pub fn format_month_year(&self, date: NaiveDate) -> String {
        match self {
            Language::Ko => format!("{}년 {:02}월", date.year(), date.month()),
            Language::En => date.format("%B %Y").to_string(),
        }
    }

    pub fn format_date_long(&self, date: NaiveDate) -> String {
        match self {
            Language::Ko => {
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
            Language::En => date.format("%A, %B %d, %Y").to_string(),
        }
    }

    // Sidebar titles
    pub fn sidebar_calendars(&self) -> &'static str {
        match self {
            Language::Ko => " 내 캘린더 ",
            Language::En => " My Calendars ",
        }
    }

    pub fn sidebar_quick_nav(&self) -> &'static str {
        match self {
            Language::Ko => "빠른 탐색:",
            Language::En => "Quick Navigation:",
        }
    }

    pub fn sidebar_tips(&self) -> Vec<&'static str> {
        match self {
            Language::Ko => vec![
                " • 탭 클릭: 뷰 전환",
                " • 셀 클릭: 일정 추가",
                " • 일정 클릭: 상세/수정",
                " • [x] 클릭: 표시 토글",
                " • L 키: 언어 전환",
            ],
            Language::En => vec![
                " • Click tab: switch view",
                " • Click slot: add event",
                " • Click event: details/edit",
                " • [x] Checkbox: toggle cal",
                " • 'L' key: toggle language",
            ],
        }
    }

    pub fn sidebar_keys(&self) -> &'static str {
        match self {
            Language::Ko => "단축키:",
            Language::En => "Keys:",
        }
    }

    pub fn sidebar_key_tips(&self) -> Vec<&'static str> {
        match self {
            Language::Ko => vec![
                " • h/j/k/l: 탐색",
                " • c/n: 새 일정",
                " • r: 실시간 동기화",
                " • A: 계정 연동 설정",
                " • t: 오늘로 점프",
                " • q: 닫기 / 종료",
            ],
            Language::En => vec![
                " • h/j/k/l: Navigate",
                " • c/n: New event",
                " • r: Sync now",
                " • A: Accounts modal",
                " • t: Jump to today",
                " • q: Close / Quit",
            ],
        }
    }

    // Modals
    pub fn modal_new_event(&self) -> &'static str {
        match self {
            Language::Ko => " 새 일정 등록 ",
            Language::En => " New Event ",
        }
    }

    pub fn modal_edit_event(&self) -> &'static str {
        match self {
            Language::Ko => " 일정 수정 ",
            Language::En => " Edit Event ",
        }
    }

    pub fn modal_event_details(&self) -> &'static str {
        match self {
            Language::Ko => " 일정 상세 정보 ",
            Language::En => " Event Details ",
        }
    }

    pub fn field_title(&self) -> &'static str {
        match self {
            Language::Ko => " 제목:        ",
            Language::En => " Title:       ",
        }
    }

    pub fn field_calendar(&self) -> &'static str {
        match self {
            Language::Ko => " 캘린더:      ",
            Language::En => " Calendar:    ",
        }
    }

    pub fn field_date(&self) -> &'static str {
        match self {
            Language::Ko => " 날짜:        ",
            Language::En => " Date:        ",
        }
    }

    pub fn field_start_time(&self) -> &'static str {
        match self {
            Language::Ko => " 시작 시간:   ",
            Language::En => " Start Time:  ",
        }
    }

    pub fn field_end_time(&self) -> &'static str {
        match self {
            Language::Ko => " 종료 시간:   ",
            Language::En => " End Time:    ",
        }
    }

    pub fn field_all_day(&self) -> &'static str {
        match self {
            Language::Ko => " 하루 종일:   ",
            Language::En => " All Day:     ",
        }
    }

    pub fn field_location(&self) -> &'static str {
        match self {
            Language::Ko => " 장소:        ",
            Language::En => " Location:    ",
        }
    }

    pub fn field_description(&self) -> &'static str {
        match self {
            Language::Ko => " 메모:        ",
            Language::En => " Description: ",
        }
    }

    pub fn placeholder_title(&self) -> &'static str {
        match self {
            Language::Ko => "(일정 제목을 입력하세요...)",
            Language::En => "(Type event title...)",
        }
    }

    pub fn placeholder_location(&self) -> &'static str {
        match self {
            Language::Ko => "(선택 입력: 회의실, 주소 등)",
            Language::En => "(Optional location)",
        }
    }

    pub fn placeholder_description(&self) -> &'static str {
        match self {
            Language::Ko => "(상세 메모를 입력하세요)",
            Language::En => "(Optional notes)",
        }
    }

    // Status messages
    pub fn msg_welcome(&self) -> &'static str {
        match self {
            Language::Ko => "rcal에 오신 것을 환영합니다! '?'를 누르거나 [ ? 도움말 ]을 클릭하세요.",
            Language::En => "Welcome to rcal! Press ? or click [ Help ] for shortcuts.",
        }
    }

    pub fn msg_saved(&self) -> &'static str {
        match self {
            Language::Ko => "일정이 성공적으로 저장되었습니다.",
            Language::En => "Event saved successfully.",
        }
    }

    pub fn msg_deleted(&self) -> &'static str {
        match self {
            Language::Ko => "일정이 삭제되었습니다.",
            Language::En => "Event deleted.",
        }
    }

    pub fn msg_jump_today(&self, date_str: &str) -> String {
        match self {
            Language::Ko => format!("오늘로 이동했습니다 ({})", date_str),
            Language::En => format!("Jumped to today ({})", date_str),
        }
    }

    pub fn msg_lang_changed(&self) -> String {
        match self {
            Language::Ko => "언어가 '한국어'로 변경되었습니다.".to_string(),
            Language::En => "Language changed to 'English'.".to_string(),
        }
    }
}
