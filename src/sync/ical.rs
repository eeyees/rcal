use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

use crate::model::{Event, SyncStatus};

pub fn event_to_ics(event: &Event) -> String {
    let now_utc = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let uid = if let Some(ref r_id) = event.remote_id {
        r_id.clone()
    } else {
        event.id.clone()
    };

    let (dtstart, dtend) = if event.all_day {
        let s = event.start_time.format("%Y%m%d").to_string();
        let e = (event.end_time + chrono::Duration::days(1)).format("%Y%m%d").to_string();
        (
            format!("DTSTART;VALUE=DATE:{}", s),
            format!("DTEND;VALUE=DATE:{}", e),
        )
    } else {
        let s = event.start_time.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string();
        let e = event.end_time.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string();
        (
            format!("DTSTART:{}", s),
            format!("DTEND:{}", e),
        )
    };

    let summary = escape_ics_text(&event.title);
    let desc_line = event.description.as_ref()
        .map(|d| format!("\r\nDESCRIPTION:{}", escape_ics_text(d)))
        .unwrap_or_default();
    let loc_line = event.location.as_ref()
        .map(|l| format!("\r\nLOCATION:{}", escape_ics_text(l)))
        .unwrap_or_default();
    let rrule_line = event.rrule.as_ref()
        .map(|r| format!("\r\nRRULE:{}", r))
        .unwrap_or_default();

    format!(
        "BEGIN:VCALENDAR\r\n\
        VERSION:2.0\r\n\
        PRODID:-//rcal//rcal calendar//EN\r\n\
        CALSCALE:GREGORIAN\r\n\
        BEGIN:VEVENT\r\n\
        UID:{}\r\n\
        DTSTAMP:{}\r\n\
        {}\r\n\
        {}\r\n\
        SUMMARY:{}{}{}{}\r\n\
        END:VEVENT\r\n\
        END:VCALENDAR\r\n",
        uid, now_utc, dtstart, dtend, summary, desc_line, loc_line, rrule_line
    )
}

pub fn ics_to_events(
    ics_str: &str,
    calendar_id: &str,
    _remote_url: &str,
    etag: Option<&str>,
) -> Vec<Event> {
    let mut events = Vec::new();
    let lines = unfold_ics_lines(ics_str);

    let mut in_vevent = false;
    let mut uid: Option<String> = None;
    let mut summary: Option<String> = None;
    let mut description: Option<String> = None;
    let mut location: Option<String> = None;
    let mut dtstart: Option<(DateTime<Local>, bool)> = None;
    let mut dtend: Option<DateTime<Local>> = None;
    let mut rrule: Option<String> = None;

    for line in lines {
        let line = line.trim();
        if line.eq_ignore_ascii_case("BEGIN:VEVENT") {
            in_vevent = true;
            uid = None;
            summary = None;
            description = None;
            location = None;
            dtstart = None;
            dtend = None;
            rrule = None;
            continue;
        }

        if line.eq_ignore_ascii_case("END:VEVENT") {
            if in_vevent {
                if let (Some((start_time, all_day)), Some(u)) = (dtstart, uid.take()) {
                    let end_time = match dtend {
                        Some(e) => {
                            if all_day && e > start_time {
                                e - chrono::Duration::days(1)
                            } else {
                                e
                            }
                        }
                        None => {
                            if all_day {
                                start_time
                            } else {
                                start_time + chrono::Duration::hours(1)
                            }
                        }
                    };

                    let title = summary.take().unwrap_or_else(|| "(No title)".to_string());
                    let ev = Event {
                        id: uuid::Uuid::new_v4().to_string(),
                        calendar_id: calendar_id.to_string(),
                        title,
                        description: description.take(),
                        location: location.take(),
                        start_time,
                        end_time,
                        all_day,
                        rrule: rrule.take(),
                        sync_status: SyncStatus::Synced,
                        remote_id: Some(u),
                        etag: etag.map(|s| s.trim_matches('"').to_string()),
                        updated_at: Local::now(),
                    };
                    events.push(ev);
                }
            }
            in_vevent = false;
            continue;
        }

        if !in_vevent {
            continue;
        }

        if let Some((key, val, key_part)) = parse_property_line(line) {
            let upper_key = key.to_ascii_uppercase();
            if upper_key == "UID" {
                uid = Some(val.to_string());
            } else if upper_key == "SUMMARY" {
                summary = Some(unescape_ics_text(val));
            } else if upper_key == "DESCRIPTION" {
                description = Some(unescape_ics_text(val));
            } else if upper_key == "LOCATION" {
                location = Some(unescape_ics_text(val));
            } else if upper_key.starts_with("DTSTART") {
                if let Some(parsed) = parse_ics_datetime(key_part, val) {
                    dtstart = Some(parsed);
                }
            } else if upper_key.starts_with("DTEND") {
                if let Some((parsed, _)) = parse_ics_datetime(key_part, val) {
                    dtend = Some(parsed);
                }
            } else if upper_key == "RRULE" {
                rrule = Some(val.to_string());
            }
        }
    }

    events
}

fn unfold_ics_lines(input: &str) -> Vec<String> {
    let mut unfolded: Vec<String> = Vec::new();
    for line in input.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(last) = unfolded.last_mut() {
                last.push_str(&line[1..]);
                continue;
            }
        }
        unfolded.push(line.to_string());
    }
    unfolded
}

pub fn parse_property_line(line: &str) -> Option<(&str, &str, &str)> {
    let idx = line.find(':')?;
    let key_part = &line[..idx];
    let val_part = &line[idx + 1..];
    let key = key_part.split(';').next().unwrap_or(key_part);
    Some((key, val_part, key_part))
}

pub fn parse_tzid(raw_tzid: &str) -> Option<Tz> {
    let tzid = raw_tzid.trim().trim_matches('"').trim_matches('\'');
    if let Ok(tz) = tzid.parse::<Tz>() {
        return Some(tz);
    }

    // Try stripping leading path (e.g. "/mozilla.org/20050126_1/Asia/Seoul" -> "Asia/Seoul")
    if tzid.contains('/') {
        let parts: Vec<&str> = tzid.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            let last_two = format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1]);
            if let Ok(tz) = last_two.parse::<Tz>() {
                return Some(tz);
            }
        }
        if let Some(last) = parts.last() {
            if let Ok(tz) = last.parse::<Tz>() {
                return Some(tz);
            }
        }
    }

    // Common abbreviations / aliases
    match tzid.to_ascii_uppercase().as_str() {
        "KST" => "Asia/Seoul".parse::<Tz>().ok(),
        "JST" => "Asia/Tokyo".parse::<Tz>().ok(),
        "UTC" | "GMT" => "UTC".parse::<Tz>().ok(),
        "EST" | "EDT" => "America/New_York".parse::<Tz>().ok(),
        "CST" | "CDT" => "America/Chicago".parse::<Tz>().ok(),
        "MST" | "MDT" => "America/Denver".parse::<Tz>().ok(),
        "PST" | "PDT" => "America/Los_Angeles".parse::<Tz>().ok(),
        _ => None,
    }
}

pub fn parse_ics_datetime(key_part: &str, val: &str) -> Option<(DateTime<Local>, bool)> {
    let val = val.trim();
    let upper_key = key_part.to_ascii_uppercase();

    // 1. All-day date only: YYYYMMDD (8 chars) or VALUE=DATE
    if (val.len() == 8 && val.chars().all(|c| c.is_ascii_digit())) || upper_key.contains("VALUE=DATE") {
        let date_str = if val.len() >= 8 { &val[0..8] } else { val };
        if date_str.len() == 8 && date_str.chars().all(|c| c.is_ascii_digit()) {
            let year = date_str[0..4].parse::<i32>().ok()?;
            let month = date_str[4..6].parse::<u32>().ok()?;
            let day = date_str[6..8].parse::<u32>().ok()?;
            let naive_date = NaiveDate::from_ymd_opt(year, month, day)?;
            let dt = Local.from_local_datetime(&naive_date.and_hms_opt(0, 0, 0)?).single()?;
            return Some((dt, true));
        }
    }

    // 2. UTC Date Time: YYYYMMDDTHHMMSSZ
    if val.ends_with('Z') && val.contains('T') {
        let s = val.trim_end_matches('Z');
        let naive = NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()?;
        let utc_dt = Utc.from_utc_datetime(&naive);
        return Some((utc_dt.with_timezone(&Local), false));
    }

    // 3. Timezone specified in parameters (e.g. DTSTART;TZID=Asia/Seoul:20261031T090000)
    let mut tzid_param: Option<&str> = None;
    for param in key_part.split(';') {
        let p = param.trim();
        if let Some(rest) = p.strip_prefix("TZID=") {
            tzid_param = Some(rest.trim());
        } else if let Some(rest) = p.strip_prefix("tzid=") {
            tzid_param = Some(rest.trim());
        }
    }

    if let Some(tzid_str) = tzid_param {
        if let Some(tz) = parse_tzid(tzid_str) {
            if val.contains('T') {
                if let Ok(naive) = NaiveDateTime::parse_from_str(val, "%Y%m%dT%H%M%S") {
                    if let Some(tz_dt) = tz.from_local_datetime(&naive).single().or_else(|| tz.from_local_datetime(&naive).earliest()) {
                        return Some((tz_dt.with_timezone(&Local), false));
                    }
                }
            }
        }
    }

    // 4. Local/Floating Date Time: YYYYMMDDTHHMMSS
    if val.contains('T') {
        if let Ok(naive) = NaiveDateTime::parse_from_str(val, "%Y%m%dT%H%M%S") {
            if let Some(dt) = Local.from_local_datetime(&naive).single().or_else(|| Local.from_local_datetime(&naive).earliest()) {
                return Some((dt, false));
            }
        }
    }

    None
}

fn escape_ics_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn unescape_ics_text(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\,", ",")
        .replace("\\;", ";")
        .replace("\\\\", "\\")
}
