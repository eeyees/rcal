use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HolidayRegion {
    None,
    #[default]
    Kr,
    Us,
    Jp,
}

impl HolidayRegion {
    pub const ALL: &'static [HolidayRegion] = &[
        HolidayRegion::Kr,
        HolidayRegion::Us,
        HolidayRegion::Jp,
        HolidayRegion::None,
    ];

    pub fn code(&self) -> &'static str {
        match self {
            HolidayRegion::None => "none",
            HolidayRegion::Kr => "kr",
            HolidayRegion::Us => "us",
            HolidayRegion::Jp => "jp",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code.to_lowercase().as_str() {
            "kr" | "korea" | "south_korea" => HolidayRegion::Kr,
            "us" | "usa" | "united_states" => HolidayRegion::Us,
            "jp" | "japan" => HolidayRegion::Jp,
            _ => HolidayRegion::None,
        }
    }

    pub fn label(&self, lang: crate::i18n::Language) -> &'static str {
        match self {
            HolidayRegion::Kr => match lang {
                crate::i18n::Language::Ko => "대한민국 (South Korea)",
                crate::i18n::Language::Ja => "韓国 (South Korea)",
                _ => "South Korea (KR)",
            },
            HolidayRegion::Us => match lang {
                crate::i18n::Language::Ko => "미국 (United States)",
                crate::i18n::Language::Ja => "アメリカ (United States)",
                _ => "United States (US)",
            },
            HolidayRegion::Jp => match lang {
                crate::i18n::Language::Ko => "일본 (Japan)",
                crate::i18n::Language::Ja => "日本 (Japan)",
                _ => "Japan (JP)",
            },
            HolidayRegion::None => match lang {
                crate::i18n::Language::Ko => "사용 안 함 (Off)",
                crate::i18n::Language::Ja => "オフ (Off)",
                _ => "Off (None)",
            },
        }
    }

    pub fn default_from_locale() -> Self {
        if let Some(locale) = sys_locale::get_locale() {
            let lower = locale.to_lowercase();
            if lower.contains("kr") || lower.contains("ko") {
                return HolidayRegion::Kr;
            } else if lower.contains("jp") || lower.contains("ja") {
                return HolidayRegion::Jp;
            } else if lower.contains("us") || lower.contains("en") {
                return HolidayRegion::Us;
            }
        }
        HolidayRegion::Kr
    }

    pub fn ical_feed_url(&self) -> Option<&'static str> {
        match self {
            HolidayRegion::Kr => Some(
                "https://calendar.google.com/calendar/ical/ko.south_korea%23holiday%40group.v.calendar.google.com/public/basic.ics",
            ),
            HolidayRegion::Us => Some(
                "https://calendar.google.com/calendar/ical/en.usa%23holiday%40group.v.calendar.google.com/public/basic.ics",
            ),
            HolidayRegion::Jp => Some(
                "https://calendar.google.com/calendar/ical/ja.japanese%23holiday%40group.v.calendar.google.com/public/basic.ics",
            ),
            HolidayRegion::None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holiday {
    pub date: NaiveDate,
    pub name: String,
}

impl Holiday {
    pub fn new(year: i32, month: u32, day: u32, name: impl Into<String>) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, month, day).map(|date| Holiday {
            date,
            name: name.into(),
        })
    }
}

/// N번째 특정 요일 찾기 (예: 5월의 마지막 월요일, 1월의 3번째 월요일)
fn nth_weekday_of_month(year: i32, month: u32, nth: u32, target_wd: Weekday) -> Option<NaiveDate> {
    let mut count = 0;
    for day in 1..=31 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date.weekday() == target_wd {
                count += 1;
                if count == nth {
                    return Some(date);
                }
            }
        }
    }
    None
}

/// 특정 달의 마지막 특정 요일 찾기
fn last_weekday_of_month(year: i32, month: u32, target_wd: Weekday) -> Option<NaiveDate> {
    for day in (22..=31).rev() {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            if date.weekday() == target_wd {
                return Some(date);
            }
        }
    }
    None
}

/// 한국 공휴일 생성 (2024 ~ 2028년 정밀 음력/대체공휴일 및 일반 규칙)
fn korean_holidays(year: i32) -> Vec<Holiday> {
    let mut holidays = Vec::new();

    // 1. 양력 고정 공휴일
    let fixed = [
        (1, 1, "신정 (New Year's Day)"),
        (3, 1, "삼일절 (3·1 Movement Day)"),
        (5, 5, "어린이날 (Children's Day)"),
        (6, 6, "현충일 (Memorial Day)"),
        (8, 15, "광복절 (Liberation Day)"),
        (10, 3, "개천절 (National Foundation Day)"),
        (10, 9, "한글날 (Hangul Day)"),
        (12, 25, "성탄절 (Christmas Day)"),
    ];

    for (m, d, name) in fixed {
        if let Some(h) = Holiday::new(year, m, d, name) {
            holidays.push(h);
        }
    }

    // 2. 음력 기반 명절 (설날, 추석, 부처님오신날) 및 대체공휴일 (2024~2028)
    match year {
        2024 => {
            holidays.extend([
                Holiday::new(2024, 2, 9, "설날 연휴").unwrap(),
                Holiday::new(2024, 2, 10, "설날").unwrap(),
                Holiday::new(2024, 2, 11, "설날 연휴").unwrap(),
                Holiday::new(2024, 2, 12, "대체공휴일 (설날)").unwrap(),
                Holiday::new(2024, 5, 6, "대체공휴일 (어린이날)").unwrap(),
                Holiday::new(2024, 5, 15, "부처님오신날 (Buddha's Birthday)").unwrap(),
                Holiday::new(2024, 9, 16, "추석 연휴").unwrap(),
                Holiday::new(2024, 9, 17, "추석").unwrap(),
                Holiday::new(2024, 9, 18, "추석 연휴").unwrap(),
            ]);
        }
        2025 => {
            holidays.extend([
                Holiday::new(2025, 1, 28, "설날 연휴").unwrap(),
                Holiday::new(2025, 1, 29, "설날").unwrap(),
                Holiday::new(2025, 1, 30, "설날 연휴").unwrap(),
                Holiday::new(2025, 3, 3, "대체공휴일 (삼일절)").unwrap(),
                Holiday::new(2025, 5, 6, "대체공휴일 (부처님오신날)").unwrap(),
                Holiday::new(2025, 10, 5, "추석 연휴").unwrap(),
                Holiday::new(2025, 10, 6, "추석").unwrap(),
                Holiday::new(2025, 10, 7, "추석 연휴").unwrap(),
                Holiday::new(2025, 10, 8, "대체공휴일 (추석)").unwrap(),
            ]);
        }
        2026 => {
            holidays.extend([
                Holiday::new(2026, 2, 16, "설날 연휴").unwrap(),
                Holiday::new(2026, 2, 17, "설날").unwrap(),
                Holiday::new(2026, 2, 18, "설날 연휴").unwrap(),
                Holiday::new(2026, 3, 2, "대체공휴일 (삼일절)").unwrap(),
                Holiday::new(2026, 5, 24, "부처님오신날 (Buddha's Birthday)").unwrap(),
                Holiday::new(2026, 5, 25, "대체공휴일 (부처님오신날)").unwrap(),
                Holiday::new(2026, 8, 17, "대체공휴일 (광복절)").unwrap(),
                Holiday::new(2026, 9, 24, "추석 연휴").unwrap(),
                Holiday::new(2026, 9, 25, "추석").unwrap(),
                Holiday::new(2026, 9, 26, "추석 연휴").unwrap(),
                Holiday::new(2026, 10, 5, "대체공휴일 (개천절)").unwrap(),
            ]);
        }
        2027 => {
            holidays.extend([
                Holiday::new(2027, 2, 6, "설날 연휴").unwrap(),
                Holiday::new(2027, 2, 7, "설날").unwrap(),
                Holiday::new(2027, 2, 8, "설날 연휴").unwrap(),
                Holiday::new(2027, 2, 9, "대체공휴일 (설날)").unwrap(),
                Holiday::new(2027, 5, 13, "부처님오신날 (Buddha's Birthday)").unwrap(),
                Holiday::new(2027, 8, 16, "대체공휴일 (광복절)").unwrap(),
                Holiday::new(2027, 9, 14, "추석 연휴").unwrap(),
                Holiday::new(2027, 9, 15, "추석").unwrap(),
                Holiday::new(2027, 9, 16, "추석 연휴").unwrap(),
                Holiday::new(2027, 10, 4, "대체공휴일 (개천절)").unwrap(),
                Holiday::new(2027, 10, 11, "대체공휴일 (한글날)").unwrap(),
            ]);
        }
        2028 => {
            holidays.extend([
                Holiday::new(2028, 1, 26, "설날 연휴").unwrap(),
                Holiday::new(2028, 1, 27, "설날").unwrap(),
                Holiday::new(2028, 1, 28, "설날 연휴").unwrap(),
                Holiday::new(2028, 5, 2, "부처님오신날 (Buddha's Birthday)").unwrap(),
                Holiday::new(2028, 10, 2, "추석 연휴").unwrap(),
                Holiday::new(2028, 10, 3, "추석").unwrap(),
                Holiday::new(2028, 10, 4, "추석 연휴").unwrap(),
                Holiday::new(2028, 10, 5, "대체공휴일 (추석/개천절)").unwrap(),
            ]);
        }
        _ => {}
    }

    // 대체공휴일 일반 보정 (삼일절, 어린이날, 광복절, 개천절, 한글날, 성탄절이 주말이면 익일 대체)
    // 2024~2028 외 연도 기본 지원
    if year < 2024 || year > 2028 {
        let targets = [(3, 1, "삼일절"), (5, 5, "어린이날"), (8, 15, "광복절"), (10, 3, "개천절"), (10, 9, "한글날"), (12, 25, "성탄절")];
        for (m, d, base_name) in targets {
            if let Some(date) = NaiveDate::from_ymd_opt(year, m, d) {
                if date.weekday() == Weekday::Sun {
                    if let Some(sub) = date.succ_opt() {
                        holidays.push(Holiday {
                            date: sub,
                            name: format!("대체공휴일 ({})", base_name),
                        });
                    }
                } else if date.weekday() == Weekday::Sat {
                    if let Some(sub) = date.checked_add_signed(chrono::Duration::days(2)) {
                        holidays.push(Holiday {
                            date: sub,
                            name: format!("대체공휴일 ({})", base_name),
                        });
                    }
                }
            }
        }
    }

    holidays.sort_by_key(|h| h.date);
    holidays.dedup_by_key(|h| h.date);
    holidays
}

/// 미국 연방 공휴일 (US Federal Holidays)
fn us_holidays(year: i32) -> Vec<Holiday> {
    let mut holidays = Vec::new();

    // 1. 고정일 (주말 보정: 토요일이면 금요일, 일요일이면 월요일)
    let fixed = [
        (1, 1, "New Year's Day"),
        (6, 19, "Juneteenth National Independence Day"),
        (7, 4, "Independence Day"),
        (11, 11, "Veterans Day"),
        (12, 25, "Christmas Day"),
    ];

    for (m, d, name) in fixed {
        if let Some(date) = NaiveDate::from_ymd_opt(year, m, d) {
            holidays.push(Holiday {
                date,
                name: name.to_string(),
            });
            // Observed holiday
            if date.weekday() == Weekday::Sun {
                if let Some(obs) = date.succ_opt() {
                    holidays.push(Holiday {
                        date: obs,
                        name: format!("{} (Observed)", name),
                    });
                }
            } else if date.weekday() == Weekday::Sat {
                if let Some(obs) = date.pred_opt() {
                    holidays.push(Holiday {
                        date: obs,
                        name: format!("{} (Observed)", name),
                    });
                }
            }
        }
    }

    // 2. 가변일
    // MLK Day: 1월 3번째 월요일
    if let Some(d) = nth_weekday_of_month(year, 1, 3, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "Martin Luther King Jr. Day".to_string(),
        });
    }
    // Presidents' Day: 2월 3번째 월요일
    if let Some(d) = nth_weekday_of_month(year, 2, 3, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "Washington's Birthday (Presidents' Day)".to_string(),
        });
    }
    // Memorial Day: 5월 마지막 월요일
    if let Some(d) = last_weekday_of_month(year, 5, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "Memorial Day".to_string(),
        });
    }
    // Labor Day: 9월 1번째 월요일
    if let Some(d) = nth_weekday_of_month(year, 9, 1, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "Labor Day".to_string(),
        });
    }
    // Columbus Day: 10월 2번째 월요일
    if let Some(d) = nth_weekday_of_month(year, 10, 2, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "Columbus Day".to_string(),
        });
    }
    // Thanksgiving Day: 11월 4번째 목요일
    if let Some(d) = nth_weekday_of_month(year, 11, 4, Weekday::Thu) {
        holidays.push(Holiday {
            date: d,
            name: "Thanksgiving Day".to_string(),
        });
    }

    holidays.sort_by_key(|h| h.date);
    holidays.dedup_by_key(|h| h.date);
    holidays
}

/// 일본 공휴일 (Japanese Public Holidays / 祝日)
fn japanese_holidays(year: i32) -> Vec<Holiday> {
    let mut holidays = Vec::new();

    let fixed = [
        (1, 1, "元日 (New Year's Day)"),
        (2, 11, "建国記念の日 (National Foundation Day)"),
        (2, 23, "天皇誕生日 (Emperor's Birthday)"),
        (4, 29, "昭和の日 (Showa Day)"),
        (5, 3, "憲法記念日 (Constitution Memorial Day)"),
        (5, 4, "みどりの日 (Greenery Day)"),
        (5, 5, "こどもの日 (Children's Day)"),
        (8, 11, "山の日 (Mountain Day)"),
        (11, 3, "文化の日 (Culture Day)"),
        (11, 23, "勤労感謝の日 (Labor Thanksgiving Day)"),
    ];

    for (m, d, name) in fixed {
        if let Some(h) = Holiday::new(year, m, d, name) {
            holidays.push(h);
        }
    }

    // 가변일 (해피 먼데이 제도)
    if let Some(d) = nth_weekday_of_month(year, 1, 2, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "成人の日 (Coming of Age Day)".to_string(),
        });
    }
    if let Some(d) = nth_weekday_of_month(year, 7, 3, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "海の日 (Marine Day)".to_string(),
        });
    }
    if let Some(d) = nth_weekday_of_month(year, 9, 3, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "敬老の日 (Respect for the Aged Day)".to_string(),
        });
    }
    if let Some(d) = nth_weekday_of_month(year, 10, 2, Weekday::Mon) {
        holidays.push(Holiday {
            date: d,
            name: "スポーツの日 (Sports Day)".to_string(),
        });
    }

    // 춘분/추분 (간이 천문 계산 근사)
    let vernal_day = if year % 4 == 0 { 20 } else { 20 };
    if let Some(h) = Holiday::new(year, 3, vernal_day, "春分の日 (Vernal Equinox Day)") {
        holidays.push(h);
    }
    let autumn_day = if (year - 2024) % 4 == 0 { 22 } else { 23 };
    if let Some(h) = Holiday::new(year, 9, autumn_day, "秋分の日 (Autumnal Equinox Day)") {
        holidays.push(h);
    }

    // 振替休日 (일요일과 겹치는 경우 다음날 휴일)
    let mut substitutes = Vec::new();
    for h in &holidays {
        if h.date.weekday() == Weekday::Sun {
            if let Some(next) = h.date.succ_opt() {
                substitutes.push(Holiday {
                    date: next,
                    name: format!("振替休日 ({})", h.name),
                });
            }
        }
    }
    holidays.extend(substitutes);

    holidays.sort_by_key(|h| h.date);
    holidays.dedup_by_key(|h| h.date);
    holidays
}

/// 특정 국가/지역 및 연도에 대한 공휴일 목록 반환
pub fn get_holidays(region: HolidayRegion, year: i32) -> Vec<Holiday> {
    match region {
        HolidayRegion::None => Vec::new(),
        HolidayRegion::Kr => korean_holidays(year),
        HolidayRegion::Us => us_holidays(year),
        HolidayRegion::Jp => japanese_holidays(year),
    }
}

/// 특정 연도 범위(예: year - 1 ..= year + 1)의 모든 공휴일 반환
pub fn get_holidays_range(region: HolidayRegion, start_year: i32, end_year: i32) -> Vec<Holiday> {
    let mut all = Vec::new();
    for y in start_year..=end_year {
        all.extend(get_holidays(region, y));
    }
    all.sort_by_key(|h| h.date);
    all.dedup_by_key(|h| h.date);
    all
}
