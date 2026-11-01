use chrono::{DateTime, TimeZone, Utc};
use rcal::model::{AuthType, CalDavAccount, ProviderType};
use rcal::storage::Database;
use rcal::sync::caldav::{CalDavClient, RemoteCalendarInfo};
use rcal::sync::ical::{ics_to_events, parse_ics_datetime, parse_tzid};
use url::Url;

#[test]
fn test_naver_username_and_url_helpers() {
    // 1. clean_username strips @naver.com
    assert_eq!(
        CalDavClient::clean_username("testuser@naver.com"),
        "testuser"
    );
    assert_eq!(CalDavClient::clean_username("testuser"), "testuser");
    assert_eq!(
        CalDavClient::clean_username("  user.name@naver.com  "),
        "user.name"
    );

    // 2. get_principal_url_for_naver
    assert_eq!(
        CalDavClient::get_principal_url_for_naver("myuser@naver.com"),
        "/principals/users/myuser"
    );
    assert_eq!(
        CalDavClient::get_principal_url_for_naver("myuser"),
        "/principals/users/myuser"
    );

    // 3. is_naver_host
    let naver_url = Url::parse("https://caldav.calendar.naver.com/").unwrap();
    assert!(CalDavClient::is_naver_host(&naver_url));

    let naver_sub_url = Url::parse("https://something.naver.com/caldav").unwrap();
    assert!(CalDavClient::is_naver_host(&naver_sub_url));

    let icloud_url = Url::parse("https://caldav.icloud.com/").unwrap();
    assert!(!CalDavClient::is_naver_host(&icloud_url));

    let google_url = Url::parse("https://apidata.googleusercontent.com/caldav/v2").unwrap();
    assert!(!CalDavClient::is_naver_host(&google_url));
}

#[test]
fn test_xml_principal_and_home_set_parsing() {
    let principal_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/</d:href>
    <d:propstat>
      <d:prop>
        <d:current-user-principal>
          <d:href>/principals/users/honggildong/</d:href>
        </d:current-user-principal>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    let principal = CalDavClient::parse_multistatus_principal(principal_xml);
    assert_eq!(principal.as_deref(), Some("/principals/users/honggildong/"));

    let home_set_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/principals/users/honggildong/</d:href>
    <d:propstat>
      <d:prop>
        <c:calendar-home-set>
          <d:href>/caldav/honggildong/</d:href>
        </c:calendar-home-set>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    let home_set = CalDavClient::parse_multistatus_calendar_home_set(home_set_xml);
    assert_eq!(home_set.as_deref(), Some("/caldav/honggildong/"));
}

#[test]
fn test_xml_calendar_parsing_both_empty_and_start_tags() {
    // Both <c:calendar/> (Empty tag) and <c:calendar></c:calendar> (Start/End tag)
    // plus a normal collection that is NOT a calendar
    let multistatus_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/">
  <!-- 1. Collection that is NOT a calendar (e.g. calendar home collection) -->
  <d:response>
    <d:href>/caldav/user/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>Home</d:displayname>
        <d:resourcetype>
          <d:collection/>
        </d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>

  <!-- 2. Calendar with Empty tag: <c:calendar/> -->
  <d:response>
    <d:href>/caldav/user/default/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>Personal Schedule</d:displayname>
        <d:resourcetype>
          <d:collection/>
          <c:calendar/>
        </d:resourcetype>
        <a:calendar-color>#03C75A</a:calendar-color>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>

  <!-- 3. Calendar with Start/End tag: <c:calendar></c:calendar> (Crucial for Naver CalDAV) -->
  <d:response>
    <d:href>/caldav/user/work-cal/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>Naver Work</d:displayname>
        <d:resourcetype>
          <d:collection/>
          <c:calendar></c:calendar>
        </d:resourcetype>
        <a:calendar-color>#00ACC1</a:calendar-color>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    let cals = CalDavClient::parse_multistatus_calendars_raw(multistatus_xml).unwrap();
    assert_eq!(
        cals.len(),
        2,
        "Must discover exactly 2 calendars (ignoring non-calendar collection)"
    );

    assert_eq!(
        cals[0],
        RemoteCalendarInfo {
            name: "Personal Schedule".to_string(),
            href: "/caldav/user/default/".to_string(),
            color_code: Some("#03C75A".to_string()),
            ctag: None,
        }
    );

    assert_eq!(
        cals[1],
        RemoteCalendarInfo {
            name: "Naver Work".to_string(),
            href: "/caldav/user/work-cal/".to_string(),
            color_code: Some("#00ACC1".to_string()),
            ctag: None,
        }
    );
}

#[test]
fn test_ics_resources_parsing_from_propfind() {
    let propfind_multistatus = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/caldav/user/cal/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/caldav/user/cal/event101.ics</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"etag-abc-123"</d:getetag>
        <d:getcontenttype>text/calendar; charset=utf-8</d:getcontenttype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/caldav/user/cal/event102.ics</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"etag-def-456"</d:getetag>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    let resources =
        CalDavClient::parse_multistatus_ics_resources(propfind_multistatus, "/caldav/user/cal/");
    assert_eq!(resources.len(), 2);
    assert_eq!(
        resources[0],
        (
            "/caldav/user/cal/event101.ics".to_string(),
            Some("etag-abc-123".to_string())
        )
    );
    assert_eq!(
        resources[1],
        (
            "/caldav/user/cal/event102.ics".to_string(),
            Some("etag-def-456".to_string())
        )
    );
}

#[test]
fn test_timezone_dtstart_parsing() {
    // 1. parse_tzid
    assert!(parse_tzid("Asia/Seoul").is_some());
    assert!(parse_tzid("\"Asia/Seoul\"").is_some());
    assert!(parse_tzid("/mozilla.org/20050126_1/Asia/Seoul").is_some());
    assert!(parse_tzid("KST").is_some());
    assert!(parse_tzid("UTC").is_some());

    // 2. DTSTART with Asia/Seoul: 2026-10-31 09:00:00 KST
    // 09:00:00 in Asia/Seoul (+09:00) is exactly 00:00:00 UTC!
    let (dt, is_all_day) =
        parse_ics_datetime("DTSTART;TZID=Asia/Seoul", "20261031T090000").unwrap();
    assert!(!is_all_day);
    let dt_utc: DateTime<Utc> = dt.with_timezone(&Utc);
    assert_eq!(dt_utc, Utc.with_ymd_and_hms(2026, 10, 31, 0, 0, 0).unwrap());

    // 3. DTEND with Asia/Seoul: 2026-10-31 10:00:00 KST = 01:00:00 UTC
    let (dt_end, _) = parse_ics_datetime("DTEND;TZID=Asia/Seoul", "20261031T100000").unwrap();
    let dt_end_utc: DateTime<Utc> = dt_end.with_timezone(&Utc);
    assert_eq!(
        dt_end_utc,
        Utc.with_ymd_and_hms(2026, 10, 31, 1, 0, 0).unwrap()
    );

    // 4. UTC time ending with Z
    let (utc_parsed, is_all_day_utc) = parse_ics_datetime("DTSTART", "20261031T090000Z").unwrap();
    assert!(!is_all_day_utc);
    assert_eq!(
        utc_parsed.with_timezone(&Utc),
        Utc.with_ymd_and_hms(2026, 10, 31, 9, 0, 0).unwrap()
    );

    // 5. All-day date (YYYYMMDD)
    let (date_parsed, is_all_day_date) =
        parse_ics_datetime("DTSTART;VALUE=DATE", "20261031").unwrap();
    assert!(is_all_day_date);
    assert_eq!(
        date_parsed.date_naive(),
        chrono::NaiveDate::from_ymd_opt(2026, 10, 31).unwrap()
    );

    // 6. Floating local time (no Z, no TZID)
    let (float_parsed, is_all_day_float) =
        parse_ics_datetime("DTSTART", "20261031T143000").unwrap();
    assert!(!is_all_day_float);
    assert_eq!(
        float_parsed.time(),
        chrono::NaiveTime::from_hms_opt(14, 30, 0).unwrap()
    );

    // 7. Full iCalendar event with Asia/Seoul timezone
    let ics = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Naver Calendar//EN\r\n\
BEGIN:VEVENT\r\n\
UID:naver-cal-event-9988\r\n\
DTSTART;TZID=Asia/Seoul:20261031T140000\r\n\
DTEND;TZID=Asia/Seoul:20261031T153000\r\n\
SUMMARY:네이버 캘린더 정기 미팅\r\n\
LOCATION:1784 사옥 12층 회의실\r\n\
DESCRIPTION:네이버 CalDAV 동기화 이슈 점검\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

    let events = ics_to_events(
        ics,
        "cal_naver_1",
        "/caldav/user/cal/9988.ics",
        Some("\"etag-555\""),
    );
    assert_eq!(events.len(), 1);
    let ev = &events[0];
    assert_eq!(ev.title, "네이버 캘린더 정기 미팅");
    assert_eq!(ev.location.as_deref(), Some("1784 사옥 12층 회의실"));
    assert_eq!(
        ev.description.as_deref(),
        Some("네이버 CalDAV 동기화 이슈 점검")
    );
    assert_eq!(ev.remote_id.as_deref(), Some("naver-cal-event-9988"));
    assert_eq!(ev.etag.as_deref(), Some("etag-555"));
    assert!(!ev.all_day);

    // 14:00:00 KST = 05:00:00 UTC
    assert_eq!(
        ev.start_time.with_timezone(&Utc),
        Utc.with_ymd_and_hms(2026, 10, 31, 5, 0, 0).unwrap()
    );
    // 15:30:00 KST = 06:30:00 UTC
    assert_eq!(
        ev.end_time.with_timezone(&Utc),
        Utc.with_ymd_and_hms(2026, 10, 31, 6, 30, 0).unwrap()
    );
}

#[test]
fn test_account_deduplication_on_save_account() {
    let db = Database::in_memory().unwrap();

    // 1. Save an initial Naver account
    let acc1 = CalDavAccount {
        id: "acc_uuid_1111".to_string(),
        name: "My Naver".to_string(),
        provider: ProviderType::Naver,
        url: "https://caldav.calendar.naver.com/".to_string(),
        username: "developer_kim".to_string(),
        password: "initial_password".to_string(),
        auth_type: AuthType::Basic,
        client_id: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        last_synced_at: None,
    };
    db.save_account(&acc1).unwrap();

    let accounts = db.get_accounts().unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "acc_uuid_1111");
    assert_eq!(accounts[0].password, "initial_password");

    // 2. Attempt to save account with same provider & username, but DIFFERENT UUID
    let acc2 = CalDavAccount {
        id: "acc_uuid_2222_different".to_string(),
        name: "My Naver Updated".to_string(),
        provider: ProviderType::Naver,
        url: "https://caldav.calendar.naver.com/".to_string(),
        username: "developer_kim".to_string(),
        password: "updated_password".to_string(),
        auth_type: AuthType::Basic,
        client_id: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        last_synced_at: None,
    };
    db.save_account(&acc2).unwrap();

    // Verify: exactly 1 account exists, original ID preserved, credentials updated!
    let accounts_after = db.get_accounts().unwrap();
    assert_eq!(
        accounts_after.len(),
        1,
        "Duplicate account row must NOT be created!"
    );
    assert_eq!(
        accounts_after[0].id, "acc_uuid_1111",
        "Existing account ID must be preserved!"
    );
    assert_eq!(accounts_after[0].name, "My Naver Updated");
    assert_eq!(accounts_after[0].password, "updated_password");

    // 3. Attempt to save with username having @naver.com appended
    let acc3 = CalDavAccount {
        id: "acc_uuid_3333_third".to_string(),
        name: "My Naver with Email".to_string(),
        provider: ProviderType::Naver,
        url: "https://caldav.calendar.naver.com/".to_string(),
        username: "developer_kim@naver.com".to_string(),
        password: "new_app_password".to_string(),
        auth_type: AuthType::Basic,
        client_id: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        last_synced_at: None,
    };
    db.save_account(&acc3).unwrap();

    let accounts_third = db.get_accounts().unwrap();
    assert_eq!(
        accounts_third.len(),
        1,
        "Must recognize developer_kim@naver.com as same account!"
    );
    assert_eq!(accounts_third[0].id, "acc_uuid_1111");
    assert_eq!(accounts_third[0].password, "new_app_password");

    // 4. Test find_account_by_provider_and_username
    let found1 = db
        .find_account_by_provider_and_username(&ProviderType::Naver, "developer_kim")
        .unwrap();
    assert!(found1.is_some());
    assert_eq!(found1.unwrap().id, "acc_uuid_1111");

    let found2 = db
        .find_account_by_provider_and_username(&ProviderType::Naver, "developer_kim@naver.com")
        .unwrap();
    assert!(found2.is_some());
    assert_eq!(found2.unwrap().id, "acc_uuid_1111");

    let not_found = db
        .find_account_by_provider_and_username(&ProviderType::Naver, "unknown_user")
        .unwrap();
    assert!(not_found.is_none());

    // 5. Test matching by url and username for Generic/Apple
    let apple_acc = CalDavAccount {
        id: "apple_uuid_1".to_string(),
        name: "Apple iCloud".to_string(),
        provider: ProviderType::Apple,
        url: "https://caldav.icloud.com/".to_string(),
        username: "steve@icloud.com".to_string(),
        password: "secret_pass_1".to_string(),
        auth_type: AuthType::Basic,
        client_id: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        last_synced_at: None,
    };
    db.save_account(&apple_acc).unwrap();

    let apple_update = CalDavAccount {
        id: "apple_uuid_2_new".to_string(),
        name: "Apple iCloud 2".to_string(),
        provider: ProviderType::Apple,
        url: "https://caldav.icloud.com/".to_string(),
        username: "steve@icloud.com".to_string(),
        password: "secret_pass_updated".to_string(),
        auth_type: AuthType::Basic,
        client_id: None,
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        last_synced_at: None,
    };
    db.save_account(&apple_update).unwrap();

    let apple_found = db
        .find_account_by_provider_and_username(&ProviderType::Apple, "steve@icloud.com")
        .unwrap()
        .unwrap();
    assert_eq!(apple_found.id, "apple_uuid_1");
    assert_eq!(apple_found.password, "secret_pass_updated");
}

#[test]
fn test_database_init_schema_cleans_up_existing_duplicate_accounts() {
    let _db = Database::in_memory().unwrap();

    // Directly insert two duplicate account rows using raw SQL
    // to simulate a database that had duplicate rows from earlier versions
    {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                url TEXT NOT NULL,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                auth_type TEXT NOT NULL DEFAULT 'basic',
                client_id TEXT,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at TEXT,
                last_synced_at TEXT
            );

            INSERT INTO accounts (id, name, provider, url, username, password)
            VALUES ('acc_dup_1', 'Naver 1', 'Naver', 'https://caldav.calendar.naver.com/', 'my_user', 'pass1');

            INSERT INTO accounts (id, name, provider, url, username, password)
            VALUES ('acc_dup_2', 'Naver 2', 'Naver', 'https://caldav.calendar.naver.com/', 'my_user', 'pass2');

            INSERT INTO accounts (id, name, provider, url, username, password)
            VALUES ('acc_other', 'Google', 'Google', 'https://apidata.googleusercontent.com/', 'guser', 'pass3');
            "#,
        ).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3);

        // Run the deduplication query that init_schema runs:
        conn.execute(
            "DELETE FROM accounts WHERE rowid NOT IN (SELECT min(rowid) FROM accounts GROUP BY provider, username)",
            [],
        ).unwrap();

        let count_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count_after, 2, "Duplicate row should have been removed");

        let remaining_id: String = conn
            .query_row(
                "SELECT id FROM accounts WHERE provider = 'Naver' AND username = 'my_user'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(remaining_id, "acc_dup_1", "First row should be preserved");
    }
}
