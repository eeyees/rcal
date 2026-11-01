#[cfg(test)]
mod tests {
    use chrono::{Local, NaiveDate, TimeZone};
    use rcal::model::rrule::{format_rrule_description, parse_rrule, rrule_matches, Freq};
    use rcal::model::Event;

    #[test]
    fn test_parse_rrule_various_formats() {
        // 1. Monthly with bymonthday
        let r1 = parse_rrule("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=12").unwrap();
        assert_eq!(r1.freq, Freq::Monthly);
        assert_eq!(r1.interval, 1);
        assert_eq!(r1.by_month_day, vec![12]);

        // 2. Yearly with bymonth only
        let r2 = parse_rrule("FREQ=YEARLY;INTERVAL=1;BYMONTH=11").unwrap();
        assert_eq!(r2.freq, Freq::Yearly);
        assert_eq!(r2.by_month, vec![11]);

        // 3. Daily with UNTIL and INTERVAL
        let r3 = parse_rrule("FREQ=DAILY;UNTIL=20101117;INTERVAL=100").unwrap();
        assert_eq!(r3.freq, Freq::Daily);
        assert_eq!(r3.interval, 100);
        assert_eq!(
            r3.until,
            Some(NaiveDate::from_ymd_opt(2010, 11, 17).unwrap())
        );

        // 4. Weekly with BYDAY
        let r4 = parse_rrule("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE,FR").unwrap();
        assert_eq!(r4.freq, Freq::Weekly);
        assert_eq!(r4.interval, 2);
        assert_eq!(r4.by_day.len(), 3);
    }

    #[test]
    fn test_naver_monthly_card_payment_days_12_13_14() {
        // Case 1: Hyundai Card payment (Day 12, started 2024-04-12)
        let rule_12 = parse_rrule("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=12").unwrap();
        let start_12 = NaiveDate::from_ymd_opt(2024, 4, 12).unwrap();
        assert!(rrule_matches(
            &rule_12,
            start_12,
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        ));
        assert!(!rrule_matches(
            &rule_12,
            start_12,
            NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
        ));
        assert!(!rrule_matches(
            &rule_12,
            start_12,
            NaiveDate::from_ymd_opt(2026, 11, 13).unwrap()
        ));

        // Case 2: Samsung Fire Insurance payment (Day 13, started 2022-09-13)
        let rule_13 = parse_rrule("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=13").unwrap();
        let start_13 = NaiveDate::from_ymd_opt(2022, 9, 13).unwrap();
        assert!(rrule_matches(
            &rule_13,
            start_13,
            NaiveDate::from_ymd_opt(2026, 11, 13).unwrap()
        ));
        assert!(!rrule_matches(
            &rule_13,
            start_13,
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        ));

        // Case 3: Shinhan Card payment (Day 14, started 2022-09-14)
        let rule_14 = parse_rrule("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=14").unwrap();
        let start_14 = NaiveDate::from_ymd_opt(2022, 9, 14).unwrap();
        assert!(rrule_matches(
            &rule_14,
            start_14,
            NaiveDate::from_ymd_opt(2026, 11, 14).unwrap()
        ));
        assert!(!rrule_matches(
            &rule_14,
            start_14,
            NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()
        ));
    }

    #[test]
    fn test_naver_yearly_birthday_day_30() {
        // Case 4: Birthday on Nov 30 (FREQ=YEARLY;INTERVAL=1;BYMONTH=11, started 2019-11-30)
        let rule_30 = parse_rrule("FREQ=YEARLY;INTERVAL=1;BYMONTH=11").unwrap();
        let start_30 = NaiveDate::from_ymd_opt(2019, 11, 30).unwrap();

        // 2026-11-30 must match
        assert!(rrule_matches(
            &rule_30,
            start_30,
            NaiveDate::from_ymd_opt(2026, 11, 30).unwrap()
        ));

        // Other dates in Nov 2026 must not match
        assert!(!rrule_matches(
            &rule_30,
            start_30,
            NaiveDate::from_ymd_opt(2026, 11, 29).unwrap()
        ));
        // Other months must not match
        assert!(!rrule_matches(
            &rule_30,
            start_30,
            NaiveDate::from_ymd_opt(2026, 10, 30).unwrap()
        ));
    }

    #[test]
    fn test_event_occurs_on_integration() {
        let start_dt = Local
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2022, 9, 14)
                    .unwrap()
                    .and_hms_opt(9, 0, 0)
                    .unwrap(),
            )
            .unwrap();
        let end_dt = Local
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2022, 9, 14)
                    .unwrap()
                    .and_hms_opt(10, 0, 0)
                    .unwrap(),
            )
            .unwrap();

        let mut event = Event::new(
            "cal_naver".to_string(),
            "신한카드 결재일".to_string(),
            start_dt,
            end_dt,
            false,
        );
        event.rrule = Some("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=14".to_string());

        // Occurs on original date in 2022
        assert!(event.occurs_on(NaiveDate::from_ymd_opt(2022, 9, 14).unwrap()));
        // Occurs on Nov 14, 2026!
        assert!(event.occurs_on(NaiveDate::from_ymd_opt(2026, 11, 14).unwrap()));
        // Does NOT occur on Nov 13 or Nov 15, 2026
        assert!(!event.occurs_on(NaiveDate::from_ymd_opt(2026, 11, 13).unwrap()));
        assert!(!event.occurs_on(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap()));
    }

    #[test]
    fn test_past_expired_recurring_event_with_until() {
        let rule = parse_rrule("FREQ=DAILY;UNTIL=20101117;INTERVAL=100").unwrap();
        let start = NaiveDate::from_ymd_opt(2009, 7, 7).unwrap();

        // 2010-10-15 (100 days after July 7 is ~Oct 15)
        let occurrence_1 = start + chrono::Duration::days(100);
        assert!(rrule_matches(&rule, start, occurrence_1));

        // Date in 2026 is far after UNTIL (2010-11-17), must return false!
        assert!(!rrule_matches(
            &rule,
            start,
            NaiveDate::from_ymd_opt(2026, 11, 12).unwrap()
        ));
    }

    #[test]
    fn test_format_rrule_description_korean() {
        let desc1 = format_rrule_description(
            "FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=12",
            rcal::i18n::Language::Ko,
        )
        .unwrap();
        assert!(desc1.contains("매월"));
        assert!(desc1.contains("12일"));

        let desc2 = format_rrule_description(
            "FREQ=YEARLY;INTERVAL=1;BYMONTH=11",
            rcal::i18n::Language::Ko,
        )
        .unwrap();
        assert!(desc2.contains("매년"));
        assert!(desc2.contains("11월"));
    }
}
