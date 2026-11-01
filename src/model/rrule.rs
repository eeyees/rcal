use chrono::{Datelike, Duration, NaiveDate, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freq {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByDayRule {
    pub ord: Option<i8>, // e.g. Some(1) for 1MO, Some(-1) for -1FR, None for MO
    pub weekday: Weekday,
}

impl ByDayRule {
    pub fn matches(&self, date: NaiveDate) -> bool {
        if date.weekday() != self.weekday {
            return false;
        }
        if let Some(target_ord) = self.ord {
            let (pos_ord, neg_ord) = nth_weekday_of_month(date);
            target_ord == pos_ord || target_ord == neg_ord
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRRule {
    pub freq: Freq,
    pub interval: u32,
    pub until: Option<NaiveDate>,
    pub count: Option<u32>,
    pub by_month: Vec<u32>,
    pub by_month_day: Vec<i32>,
    pub by_day: Vec<ByDayRule>,
    pub wkst: Weekday,
}

pub fn parse_rrule(rrule_str: &str) -> Option<ParsedRRule> {
    let mut freq: Option<Freq> = None;
    let mut interval = 1u32;
    let mut until: Option<NaiveDate> = None;
    let mut count: Option<u32> = None;
    let mut by_month = Vec::new();
    let mut by_month_day = Vec::new();
    let mut by_day = Vec::new();
    let mut wkst = Weekday::Mon;

    let parts = rrule_str.split(';');
    for part in parts {
        let mut kv = part.splitn(2, '=');
        let key = kv.next()?.trim().to_ascii_uppercase();
        let val = kv.next()?.trim();

        match key.as_str() {
            "FREQ" => {
                freq = match val.to_ascii_uppercase().as_str() {
                    "DAILY" => Some(Freq::Daily),
                    "WEEKLY" => Some(Freq::Weekly),
                    "MONTHLY" => Some(Freq::Monthly),
                    "YEARLY" => Some(Freq::Yearly),
                    _ => None,
                };
            }
            "INTERVAL" => {
                if let Ok(n) = val.parse::<u32>() {
                    interval = n.max(1);
                }
            }
            "UNTIL" => {
                // Formats: YYYYMMDD or YYYYMMDDTHHMMSSZ or YYYYMMDDTHHMMSS
                let clean_val = val.trim();
                let date_part = if clean_val.len() >= 8 {
                    &clean_val[..8]
                } else {
                    clean_val
                };
                if let Ok(d) = NaiveDate::parse_from_str(date_part, "%Y%m%d") {
                    until = Some(d);
                }
            }
            "COUNT" => {
                if let Ok(n) = val.parse::<u32>() {
                    count = Some(n);
                }
            }
            "BYMONTH" => {
                for m_str in val.split(',') {
                    if let Ok(m) = m_str.trim().parse::<u32>() {
                        if (1..=12).contains(&m) {
                            by_month.push(m);
                        }
                    }
                }
            }
            "BYMONTHDAY" => {
                for d_str in val.split(',') {
                    if let Ok(d) = d_str.trim().parse::<i32>() {
                        if d >= -31 && d <= 31 && d != 0 {
                            by_month_day.push(d);
                        }
                    }
                }
            }
            "BYDAY" => {
                for day_str in val.split(',') {
                    let d = day_str.trim().to_ascii_uppercase();
                    if d.len() >= 2 {
                        let (ord_part, code_part) = d.split_at(d.len() - 2);
                        let weekday = match code_part {
                            "MO" => Some(Weekday::Mon),
                            "TU" => Some(Weekday::Tue),
                            "WE" => Some(Weekday::Wed),
                            "TH" => Some(Weekday::Thu),
                            "FR" => Some(Weekday::Fri),
                            "SA" => Some(Weekday::Sat),
                            "SU" => Some(Weekday::Sun),
                            _ => None,
                        };
                        if let Some(wd) = weekday {
                            let ord = if ord_part.is_empty() {
                                None
                            } else {
                                ord_part.parse::<i8>().ok()
                            };
                            by_day.push(ByDayRule { ord, weekday: wd });
                        }
                    }
                }
            }
            "WKST" => {
                wkst = match val.to_ascii_uppercase().as_str() {
                    "MO" => Weekday::Mon,
                    "TU" => Weekday::Tue,
                    "WE" => Weekday::Wed,
                    "TH" => Weekday::Thu,
                    "FR" => Weekday::Fri,
                    "SA" => Weekday::Sat,
                    "SU" => Weekday::Sun,
                    _ => Weekday::Mon,
                };
            }
            _ => {}
        }
    }

    Some(ParsedRRule {
        freq: freq?,
        interval,
        until,
        count,
        by_month,
        by_month_day,
        by_day,
        wkst,
    })
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    match next_month_date {
        Some(d) => (d - Duration::days(1)).day(),
        None => 31,
    }
}

pub fn nth_weekday_of_month(date: NaiveDate) -> (i8, i8) {
    let day = date.day();
    let pos_ord = ((day - 1) / 7 + 1) as i8;
    let total_days = days_in_month(date.year(), date.month());
    let neg_ord = -(((total_days - day) / 7 + 1) as i8);
    (pos_ord, neg_ord)
}

fn week_start(d: NaiveDate, wkst: Weekday) -> NaiveDate {
    let days = (d.weekday().num_days_from_monday() + 7 - wkst.num_days_from_monday()) % 7;
    d - Duration::days(days as i64)
}

pub fn rrule_matches(rule: &ParsedRRule, orig_start: NaiveDate, target: NaiveDate) -> bool {
    if target < orig_start {
        return false;
    }
    if target == orig_start {
        return true;
    }
    if let Some(until) = rule.until {
        if target > until {
            return false;
        }
    }

    match rule.freq {
        Freq::Daily => {
            let diff = (target - orig_start).num_days();
            if diff < 0 || diff % (rule.interval as i64) != 0 {
                return false;
            }
            if !rule.by_month.is_empty() && !rule.by_month.contains(&target.month()) {
                return false;
            }
            if !rule.by_day.is_empty() && !rule.by_day.iter().any(|d| d.matches(target)) {
                return false;
            }
            if let Some(count) = rule.count {
                let occ_idx = (diff / (rule.interval as i64)) as u32;
                if occ_idx >= count {
                    return false;
                }
            }
            true
        }
        Freq::Weekly => {
            let orig_week = week_start(orig_start, rule.wkst);
            let target_week = week_start(target, rule.wkst);
            let weeks_diff = (target_week - orig_week).num_days() / 7;
            if weeks_diff < 0 || weeks_diff % (rule.interval as i64) != 0 {
                return false;
            }
            if !rule.by_day.is_empty() {
                if !rule.by_day.iter().any(|d| d.matches(target)) {
                    return false;
                }
            } else if target.weekday() != orig_start.weekday() {
                return false;
            }
            if !rule.by_month.is_empty() && !rule.by_month.contains(&target.month()) {
                return false;
            }
            if let Some(count) = rule.count {
                let occ_idx = (weeks_diff / (rule.interval as i64)) as u32;
                if occ_idx >= count {
                    return false;
                }
            }
            true
        }
        Freq::Monthly => {
            let orig_year = orig_start.year();
            let orig_month = orig_start.month() as i32;
            let target_year = target.year();
            let target_month = target.month() as i32;
            let months_diff = (target_year - orig_year) * 12 + (target_month - orig_month);
            if months_diff < 0 || months_diff % (rule.interval as i32) != 0 {
                return false;
            }
            if !rule.by_month.is_empty() && !rule.by_month.contains(&target.month()) {
                return false;
            }

            // Check day match
            if !rule.by_month_day.is_empty() {
                let target_day = target.day() as i32;
                let total_days = days_in_month(target.year(), target.month()) as i32;
                let day_matched = rule.by_month_day.iter().any(|&bmd| {
                    if bmd > 0 {
                        bmd == target_day
                    } else {
                        // Negative index: -1 is last day of month
                        (total_days + 1 + bmd) == target_day
                    }
                });
                if !day_matched {
                    return false;
                }
            } else if !rule.by_day.is_empty() {
                if !rule.by_day.iter().any(|d| d.matches(target)) {
                    return false;
                }
            } else {
                // Default: same day of month as start date
                let orig_day = orig_start.day();
                if target.day() != orig_day {
                    return false;
                }
            }

            if let Some(count) = rule.count {
                let occ_idx = (months_diff / (rule.interval as i32)) as u32;
                if occ_idx >= count {
                    return false;
                }
            }
            true
        }
        Freq::Yearly => {
            let years_diff = target.year() - orig_start.year();
            if years_diff < 0 || years_diff % (rule.interval as i32) != 0 {
                return false;
            }

            // Month check
            if !rule.by_month.is_empty() {
                if !rule.by_month.contains(&target.month()) {
                    return false;
                }
            } else if target.month() != orig_start.month() {
                return false;
            }

            // Day check
            if !rule.by_month_day.is_empty() {
                let target_day = target.day() as i32;
                let total_days = days_in_month(target.year(), target.month()) as i32;
                let day_matched = rule.by_month_day.iter().any(|&bmd| {
                    if bmd > 0 {
                        bmd == target_day
                    } else {
                        (total_days + 1 + bmd) == target_day
                    }
                });
                if !day_matched {
                    return false;
                }
            } else if !rule.by_day.is_empty() {
                if !rule.by_day.iter().any(|d| d.matches(target)) {
                    return false;
                }
            } else {
                // Default: same day of month as start date
                if target.day() != orig_start.day() {
                    return false;
                }
            }

            if let Some(count) = rule.count {
                let occ_idx = (years_diff / (rule.interval as i32)) as u32;
                if occ_idx >= count {
                    return false;
                }
            }
            true
        }
    }
}

pub fn format_rrule_description(rrule_str: &str, lang: crate::i18n::Language) -> Option<String> {
    let rule = parse_rrule(rrule_str)?;
    let is_ko = matches!(lang, crate::i18n::Language::Ko);
    let is_ja = matches!(lang, crate::i18n::Language::Ja);

    let desc = match rule.freq {
        Freq::Daily => {
            if rule.interval == 1 {
                if is_ko {
                    "매일 반복".to_string()
                } else if is_ja {
                    "毎日".to_string()
                } else {
                    "Every day".to_string()
                }
            } else if is_ko {
                format!("{}일마다 반복", rule.interval)
            } else if is_ja {
                format!("{}日ごと", rule.interval)
            } else {
                format!("Every {} days", rule.interval)
            }
        }
        Freq::Weekly => {
            let days_str = if rule.by_day.is_empty() {
                String::new()
            } else {
                let names: Vec<&str> = rule
                    .by_day
                    .iter()
                    .map(|d| match d.weekday {
                        Weekday::Mon => {
                            if is_ko {
                                "월"
                            } else if is_ja {
                                "月"
                            } else {
                                "Mon"
                            }
                        }
                        Weekday::Tue => {
                            if is_ko {
                                "화"
                            } else if is_ja {
                                "火"
                            } else {
                                "Tue"
                            }
                        }
                        Weekday::Wed => {
                            if is_ko {
                                "수"
                            } else if is_ja {
                                "水"
                            } else {
                                "Wed"
                            }
                        }
                        Weekday::Thu => {
                            if is_ko {
                                "목"
                            } else if is_ja {
                                "木"
                            } else {
                                "Thu"
                            }
                        }
                        Weekday::Fri => {
                            if is_ko {
                                "금"
                            } else if is_ja {
                                "金"
                            } else {
                                "Fri"
                            }
                        }
                        Weekday::Sat => {
                            if is_ko {
                                "토"
                            } else if is_ja {
                                "土"
                            } else {
                                "Sat"
                            }
                        }
                        Weekday::Sun => {
                            if is_ko {
                                "일"
                            } else if is_ja {
                                "日"
                            } else {
                                "Sun"
                            }
                        }
                    })
                    .collect();
                names.join(", ")
            };

            if rule.interval == 1 {
                if days_str.is_empty() {
                    if is_ko {
                        "매주 반복".to_string()
                    } else if is_ja {
                        "毎週".to_string()
                    } else {
                        "Every week".to_string()
                    }
                } else if is_ko {
                    format!("매주 ({}) 반복", days_str)
                } else if is_ja {
                    format!("毎週 ({})", days_str)
                } else {
                    format!("Every week on {}", days_str)
                }
            } else if days_str.is_empty() {
                if is_ko {
                    format!("{}주마다 반복", rule.interval)
                } else if is_ja {
                    format!("{}週間ごと", rule.interval)
                } else {
                    format!("Every {} weeks", rule.interval)
                }
            } else if is_ko {
                format!("{}주마다 ({}) 반복", rule.interval, days_str)
            } else if is_ja {
                format!("{}週間ごと ({})", rule.interval, days_str)
            } else {
                format!("Every {} weeks on {}", rule.interval, days_str)
            }
        }
        Freq::Monthly => {
            let day_info = if !rule.by_month_day.is_empty() {
                let days: Vec<String> = rule
                    .by_month_day
                    .iter()
                    .map(|d| {
                        if *d > 0 {
                            format!("{}일", d)
                        } else {
                            format!("마지막 {}일", d.abs())
                        }
                    })
                    .collect();
                days.join(", ")
            } else {
                String::new()
            };

            if rule.interval == 1 {
                if day_info.is_empty() {
                    if is_ko {
                        "매월 반복".to_string()
                    } else if is_ja {
                        "毎月".to_string()
                    } else {
                        "Every month".to_string()
                    }
                } else if is_ko {
                    format!("매월 ({}) 반복", day_info)
                } else if is_ja {
                    format!("毎月 ({})", day_info)
                } else {
                    format!("Every month on {}", day_info)
                }
            } else if day_info.is_empty() {
                if is_ko {
                    format!("{}개월마다 반복", rule.interval)
                } else if is_ja {
                    format!("{}ヶ月ごと", rule.interval)
                } else {
                    format!("Every {} months", rule.interval)
                }
            } else if is_ko {
                format!("{}개월마다 ({}) 반복", rule.interval, day_info)
            } else if is_ja {
                format!("{}ヶ月ごと ({})", rule.interval, day_info)
            } else {
                format!("Every {} months on {}", rule.interval, day_info)
            }
        }
        Freq::Yearly => {
            let month_info = if !rule.by_month.is_empty() {
                let months: Vec<String> =
                    rule.by_month.iter().map(|m| format!("{}월", m)).collect();
                months.join(", ")
            } else {
                String::new()
            };

            if rule.interval == 1 {
                if month_info.is_empty() {
                    if is_ko {
                        "매년 반복".to_string()
                    } else if is_ja {
                        "毎年".to_string()
                    } else {
                        "Every year".to_string()
                    }
                } else if is_ko {
                    format!("매년 ({}) 반복", month_info)
                } else if is_ja {
                    format!("毎年 ({})", month_info)
                } else {
                    format!("Every year in {}", month_info)
                }
            } else if month_info.is_empty() {
                if is_ko {
                    format!("{}년마다 반복", rule.interval)
                } else if is_ja {
                    format!("{}年ごと", rule.interval)
                } else {
                    format!("Every {} years", rule.interval)
                }
            } else if is_ko {
                format!("{}년마다 ({}) 반복", rule.interval, month_info)
            } else if is_ja {
                format!("{}年ごと ({})", rule.interval, month_info)
            } else {
                format!("Every {} years in {}", rule.interval, month_info)
            }
        }
    };

    Some(desc)
}
