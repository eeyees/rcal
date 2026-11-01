use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, Method, StatusCode};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCalendarInfo {
    pub name: String,
    pub href: String,
    pub color_code: Option<String>,
    pub ctag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteEventData {
    pub href: String,
    pub etag: Option<String>,
    pub ics_content: String,
}

pub struct CalDavClient {
    client: Client,
    base_url: Url,
    username: String,
    password: String,
}

impl CalDavClient {
    pub fn new(url_str: &str, username: String, password: String) -> Result<Self> {
        let mut base_url =
            Url::parse(url_str).with_context(|| format!("Invalid CalDAV URL: {}", url_str))?;

        // If path is empty, make sure it ends with /
        if base_url.path().is_empty() {
            base_url.set_path("/");
        }

        let mut headers = HeaderMap::new();
        let ua = format!("rcal/{} (Rust TUI Calendar)", crate::version());
        if let Ok(val) = HeaderValue::from_str(&ua) {
            headers.insert(USER_AGENT, val);
        }

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            base_url,
            username,
            password,
        })
    }

    /// Strip @naver.com suffix if present for Naver usernames
    pub fn clean_username(username: &str) -> &str {
        let trimmed = username.trim();
        trimmed.strip_suffix("@naver.com").unwrap_or(trimmed).trim()
    }

    /// Build standard Naver CalDAV principal URL
    pub fn get_principal_url_for_naver(username: &str) -> String {
        format!("/principals/users/{}", Self::clean_username(username))
    }

    /// Check if URL host belongs to Naver CalDAV
    pub fn is_naver_host(url: &Url) -> bool {
        url.host_str()
            .map(|h| h.contains("naver.com"))
            .unwrap_or(false)
    }

    /// Step 1: Query current-user-principal via PROPFIND Depth 0
    pub async fn find_current_user_principal(&self) -> Result<Option<String>> {
        let propfind_xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal />
  </d:prop>
</d:propfind>"#;

        let res = self
            .client
            .request(
                Method::from_bytes(b"PROPFIND").unwrap(),
                self.base_url.as_str(),
            )
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "0")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(propfind_xml)
            .send()
            .await;

        if let Ok(resp) = res {
            if resp.status().is_success() || resp.status() == StatusCode::MULTI_STATUS {
                if let Ok(text) = resp.text().await {
                    return Ok(Self::parse_multistatus_principal(&text));
                }
            }
        }

        Ok(None)
    }

    /// Step 2: Query calendar-home-set via PROPFIND Depth 0 on principal URL
    pub async fn find_calendar_home_set(&self, principal_url: &str) -> Result<Option<String>> {
        let propfind_xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <c:calendar-home-set />
  </d:prop>
</d:propfind>"#;

        let resolved = self.resolve_url(principal_url)?;
        let res = self
            .client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), resolved.as_str())
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "0")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(propfind_xml)
            .send()
            .await;

        if let Ok(resp) = res {
            if resp.status().is_success() || resp.status() == StatusCode::MULTI_STATUS {
                if let Ok(text) = resp.text().await {
                    return Ok(Self::parse_multistatus_calendar_home_set(&text));
                }
            }
        }

        Ok(None)
    }

    /// Step 3: Discover available calendars via PROPFIND Depth 1
    async fn propfind_calendars(&self, target_url: &str) -> Result<Vec<RemoteCalendarInfo>> {
        let propfind_xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/">
  <d:prop>
    <d:displayname />
    <d:resourcetype />
    <a:calendar-color />
    <c:calendar-color />
  </d:prop>
</d:propfind>"#;

        let resolved = self.resolve_url(target_url)?;
        let res = self
            .client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), resolved.as_str())
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(propfind_xml)
            .send()
            .await
            .with_context(|| format!("Failed to connect to CalDAV server at {}", resolved))?;

        if !res.status().is_success() && res.status() != StatusCode::MULTI_STATUS {
            return Err(anyhow!(
                "CalDAV PROPFIND failed with HTTP status: {}",
                res.status()
            ));
        }

        let text = res.text().await?;
        Self::parse_multistatus_calendars_raw(&text)
    }

    /// Full RFC 4791 / RFC 5397 CalDAV Discovery with Naver optimization
    pub async fn discover_calendars(&self) -> Result<Vec<RemoteCalendarInfo>> {
        let is_naver = Self::is_naver_host(&self.base_url);
        let clean_user = Self::clean_username(&self.username);

        // Step 1: Discover Principal URL
        let principal_url = if is_naver {
            if self.base_url.path().contains("/principals/")
                || self.base_url.path().contains("/caldav/")
            {
                self.base_url.path().to_string()
            } else {
                Self::get_principal_url_for_naver(&self.username)
            }
        } else {
            match self.find_current_user_principal().await {
                Ok(Some(p)) => p,
                _ => self.base_url.path().to_string(),
            }
        };

        // Step 2: Discover Calendar Home Set from Principal URL
        let home_set_url = match self.find_calendar_home_set(&principal_url).await {
            Ok(Some(home)) => home,
            _ => {
                if is_naver {
                    format!("/caldav/{}/", clean_user)
                } else {
                    principal_url.clone()
                }
            }
        };

        // Step 3: Discover Calendars from Calendar Home Set
        let target_url = if home_set_url.ends_with('/') {
            home_set_url.clone()
        } else {
            format!("{}/", home_set_url)
        };

        match self.propfind_calendars(&target_url).await {
            Ok(cals) if !cals.is_empty() => Ok(cals),
            _ => {
                // If home_set target returned empty, try querying principal_url
                if target_url != principal_url {
                    let principal_target = if principal_url.ends_with('/') {
                        principal_url.clone()
                    } else {
                        format!("{}/", principal_url)
                    };
                    if let Ok(cals) = self.propfind_calendars(&principal_target).await {
                        if !cals.is_empty() {
                            return Ok(cals);
                        }
                    }
                }

                // If Naver, also try /caldav/{clean_user}/ directly
                if is_naver {
                    let naver_direct = format!("/caldav/{}/", clean_user);
                    if target_url != naver_direct {
                        if let Ok(cals) = self.propfind_calendars(&naver_direct).await {
                            if !cals.is_empty() {
                                return Ok(cals);
                            }
                        }
                    }
                }

                // Fallback to base_url PROPFIND
                let base_path = self.base_url.path().to_string();
                if let Ok(cals) = self.propfind_calendars(&base_path).await {
                    if !cals.is_empty() {
                        return Ok(cals);
                    }
                }

                // Default fallback if no calendars discovered
                Ok(vec![RemoteCalendarInfo {
                    name: "Primary Calendar".to_string(),
                    href: self.base_url.path().to_string(),
                    color_code: Some("#4285F4".to_string()),
                    ctag: None,
                }])
            }
        }
    }

    /// Fetch events using CalDAV calendar-query REPORT, with fallback to PROPFIND Depth 1
    pub async fn fetch_events(&self, calendar_href: &str) -> Result<Vec<RemoteEventData>> {
        let calendar_url = self.resolve_url(calendar_href)?;

        // Attempt 1: CalDAV REPORT calendar-query
        match self.fetch_events_report(&calendar_url).await {
            Ok(events) if !events.is_empty() => Ok(events),
            Ok(_empty) => {
                // If REPORT returned 0 events, try PROPFIND fallback
                match self
                    .fetch_events_propfind(&calendar_url, calendar_href)
                    .await
                {
                    Ok(fallback_events) if !fallback_events.is_empty() => Ok(fallback_events),
                    _ => Ok(Vec::new()),
                }
            }
            Err(report_err) => {
                // If REPORT returned 405/501 or another error, try PROPFIND fallback
                match self
                    .fetch_events_propfind(&calendar_url, calendar_href)
                    .await
                {
                    Ok(fallback_events) => Ok(fallback_events),
                    Err(_) => Err(report_err),
                }
            }
        }
    }

    async fn fetch_events_report(&self, calendar_url: &Url) -> Result<Vec<RemoteEventData>> {
        let report_xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag />
    <c:calendar-data />
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VEVENT" />
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#;

        let res = self
            .client
            .request(
                Method::from_bytes(b"REPORT").unwrap(),
                calendar_url.as_str(),
            )
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(report_xml)
            .send()
            .await
            .with_context(|| format!("Failed to query events from {}", calendar_url))?;

        if !res.status().is_success() && res.status() != StatusCode::MULTI_STATUS {
            return Err(anyhow!(
                "CalDAV REPORT failed with HTTP status: {}",
                res.status()
            ));
        }

        let text = res.text().await?;
        Self::parse_multistatus_events(&text)
    }

    async fn fetch_events_propfind(
        &self,
        calendar_url: &Url,
        calendar_href: &str,
    ) -> Result<Vec<RemoteEventData>> {
        let propfind_xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag />
    <d:getcontenttype />
    <d:resourcetype />
    <c:calendar-data />
  </d:prop>
</d:propfind>"#;

        let res = self
            .client
            .request(
                Method::from_bytes(b"PROPFIND").unwrap(),
                calendar_url.as_str(),
            )
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(propfind_xml)
            .send()
            .await
            .with_context(|| format!("Failed to PROPFIND events from {}", calendar_url))?;

        if !res.status().is_success() && res.status() != StatusCode::MULTI_STATUS {
            return Err(anyhow!(
                "CalDAV Depth 1 PROPFIND failed with HTTP status: {}",
                res.status()
            ));
        }

        let text = res.text().await?;
        // If calendar-data is included directly in the PROPFIND response:
        let parsed = Self::parse_multistatus_events(&text)?;
        if !parsed.is_empty() {
            return Ok(parsed);
        }

        // Otherwise, extract .ics hrefs from the PROPFIND response and fetch them individually via GET
        let ics_items = Self::parse_multistatus_ics_resources(&text, calendar_href);
        let mut events = Vec::new();

        for (href, etag) in ics_items {
            if let Ok(event_url) = self.resolve_url(&href) {
                let get_res = self
                    .client
                    .get(event_url.as_str())
                    .basic_auth(&self.username, Some(&self.password))
                    .header(
                        USER_AGENT,
                        format!("rcal/{} (Rust TUI Calendar)", crate::version()),
                    )
                    .send()
                    .await;

                if let Ok(resp) = get_res {
                    if resp.status().is_success() {
                        let resp_etag = resp
                            .headers()
                            .get("ETag")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.trim_matches('"').to_string())
                            .or(etag);

                        if let Ok(ics_content) = resp.text().await {
                            if !ics_content.trim().is_empty() {
                                events.push(RemoteEventData {
                                    href,
                                    etag: resp_etag,
                                    ics_content,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    /// Upload/Create/Update event via HTTP PUT
    pub async fn upload_event(
        &self,
        calendar_href: &str,
        uid: &str,
        ics_content: &str,
        etag: Option<&str>,
    ) -> Result<Option<String>> {
        let target_url = if calendar_href.ends_with('/') {
            format!("{}{}.ics", calendar_href, uid)
        } else {
            format!("{}/{}.ics", calendar_href, uid)
        };
        let resolved_url = self.resolve_url(&target_url)?;

        let mut req = self
            .client
            .put(resolved_url.as_str())
            .basic_auth(&self.username, Some(&self.password))
            .header(CONTENT_TYPE, "text/calendar; charset=utf-8")
            .body(ics_content.to_string());

        if let Some(e) = etag {
            let quoted = if e.starts_with('"') {
                e.to_string()
            } else {
                format!("\"{}\"", e)
            };
            req = req.header("If-Match", quoted);
        }

        let res = req.send().await?;
        if !res.status().is_success() {
            return Err(anyhow!(
                "CalDAV PUT failed with HTTP status: {}",
                res.status()
            ));
        }

        let new_etag = res
            .headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        Ok(new_etag)
    }

    /// Delete event via HTTP DELETE
    pub async fn delete_event(&self, event_href: &str, etag: Option<&str>) -> Result<()> {
        let resolved_url = self.resolve_url(event_href)?;

        let mut req = self
            .client
            .delete(resolved_url.as_str())
            .basic_auth(&self.username, Some(&self.password));

        if let Some(e) = etag {
            let quoted = if e.starts_with('"') {
                e.to_string()
            } else {
                format!("\"{}\"", e)
            };
            req = req.header("If-Match", quoted);
        }

        let res = req.send().await?;
        if !res.status().is_success() && res.status() != StatusCode::NOT_FOUND {
            return Err(anyhow!(
                "CalDAV DELETE failed with HTTP status: {}",
                res.status()
            ));
        }

        Ok(())
    }

    fn resolve_url(&self, href: &str) -> Result<Url> {
        if href.starts_with("http://") || href.starts_with("https://") {
            Url::parse(href).context("Invalid absolute URL")
        } else {
            self.base_url
                .join(href)
                .context("Failed to join relative URL with base")
        }
    }

    /// Parse href from current-user-principal response
    pub fn parse_multistatus_principal(xml_text: &str) -> Option<String> {
        use quick_xml::events::Event as XmlEvent;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut in_principal = false;
        let mut in_href = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(XmlEvent::Start(ref e)) => {
                    let local = e.local_name();
                    let name = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name == "current-user-principal" {
                        in_principal = true;
                    } else if in_principal && name == "href" {
                        in_href = true;
                    }
                }
                Ok(XmlEvent::Text(ref e)) => {
                    if in_principal && in_href {
                        let text = e.unescape().unwrap_or_default().trim().to_string();
                        if !text.is_empty() {
                            return Some(text);
                        }
                    }
                }
                Ok(XmlEvent::End(ref e)) => {
                    let local = e.local_name();
                    let name = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name == "href" {
                        in_href = false;
                    } else if name == "current-user-principal" {
                        in_principal = false;
                    }
                }
                Ok(XmlEvent::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        None
    }

    /// Parse href from calendar-home-set response
    pub fn parse_multistatus_calendar_home_set(xml_text: &str) -> Option<String> {
        use quick_xml::events::Event as XmlEvent;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut in_home_set = false;
        let mut in_href = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(XmlEvent::Start(ref e)) => {
                    let local = e.local_name();
                    let name = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name == "calendar-home-set" {
                        in_home_set = true;
                    } else if in_home_set && name == "href" {
                        in_href = true;
                    }
                }
                Ok(XmlEvent::Text(ref e)) => {
                    if in_home_set && in_href {
                        let text = e.unescape().unwrap_or_default().trim().to_string();
                        if !text.is_empty() {
                            return Some(text);
                        }
                    }
                }
                Ok(XmlEvent::End(ref e)) => {
                    let local = e.local_name();
                    let name = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name == "href" {
                        in_href = false;
                    } else if name == "calendar-home-set" {
                        in_home_set = false;
                    }
                }
                Ok(XmlEvent::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        None
    }

    /// Parse calendars directly from Multi-Status XML.
    /// Supports both `<c:calendar/>` (Empty) and `<c:calendar></c:calendar>` (Start).
    pub fn parse_multistatus_calendars_raw(xml_text: &str) -> Result<Vec<RemoteCalendarInfo>> {
        let mut calendars = Vec::new();
        use quick_xml::events::Event as XmlEvent;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut in_response = false;
        let mut current_href = String::new();
        let mut current_name = String::new();
        let mut current_color: Option<String> = None;
        let mut is_calendar = false;
        let mut current_tag = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(XmlEvent::Start(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "response" {
                        in_response = true;
                        current_href.clear();
                        current_name.clear();
                        current_color = None;
                        is_calendar = false;
                    } else if in_response {
                        // Crucial fix: <calendar></calendar> as Start tag marks is_calendar = true
                        if name_str == "calendar" {
                            is_calendar = true;
                        }
                        current_tag = name_str;
                    }
                }
                Ok(XmlEvent::Empty(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    // <calendar/> as Empty tag marks is_calendar = true
                    if name_str == "calendar" {
                        is_calendar = true;
                    }
                }
                Ok(XmlEvent::Text(ref e)) => {
                    if in_response {
                        let text = e.unescape().unwrap_or_default().trim().to_string();
                        if current_tag == "href" && current_href.is_empty() {
                            current_href = text;
                        } else if current_tag == "displayname" {
                            current_name = text;
                        } else if current_tag.contains("calendar-color") {
                            current_color = Some(text);
                        } else if current_tag == "resourcetype" && text.contains("calendar") {
                            is_calendar = true;
                        }
                    }
                }
                Ok(XmlEvent::End(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "response" {
                        if in_response && is_calendar && !current_href.is_empty() {
                            let display_name = if current_name.is_empty() {
                                "Calendar".to_string()
                            } else {
                                current_name.clone()
                            };

                            calendars.push(RemoteCalendarInfo {
                                name: display_name,
                                href: current_href.clone(),
                                color_code: current_color.clone(),
                                ctag: None,
                            });
                        }
                        in_response = false;
                    }
                    current_tag.clear();
                }
                Ok(XmlEvent::Eof) => break,
                Err(err) => return Err(anyhow!("XML parse error: {}", err)),
                _ => {}
            }
            buf.clear();
        }

        Ok(calendars)
    }

    /// Parse calendars with fallback to Primary Calendar if empty
    pub fn parse_multistatus_calendars(
        base_url: &Url,
        xml_text: &str,
    ) -> Result<Vec<RemoteCalendarInfo>> {
        let mut calendars = Self::parse_multistatus_calendars_raw(xml_text)?;

        if calendars.is_empty() {
            calendars.push(RemoteCalendarInfo {
                name: "Primary Calendar".to_string(),
                href: base_url.path().to_string(),
                color_code: Some("#4285F4".to_string()),
                ctag: None,
            });
        }

        Ok(calendars)
    }

    /// Parse events from Multi-Status XML containing calendar-data
    pub fn parse_multistatus_events(xml_text: &str) -> Result<Vec<RemoteEventData>> {
        let mut events = Vec::new();
        use quick_xml::events::Event as XmlEvent;
        use quick_xml::reader::Reader;

        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(false);

        let mut in_response = false;
        let mut current_href = String::new();
        let mut current_etag: Option<String> = None;
        let mut current_ics = String::new();
        let mut current_tag = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(XmlEvent::Start(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "response" {
                        in_response = true;
                        current_href.clear();
                        current_etag = None;
                        current_ics.clear();
                    } else if in_response {
                        current_tag = name_str;
                    }
                }
                Ok(XmlEvent::Text(ref e)) => {
                    if in_response {
                        let text = e.unescape().unwrap_or_default().to_string();
                        if current_tag == "href" && current_href.is_empty() {
                            current_href = text.trim().to_string();
                        } else if current_tag == "getetag" {
                            current_etag = Some(text.trim_matches('"').to_string());
                        } else if current_tag == "calendar-data" {
                            current_ics.push_str(&text);
                        }
                    }
                }
                Ok(XmlEvent::CData(ref e)) => {
                    if in_response && current_tag == "calendar-data" {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        current_ics.push_str(&text);
                    }
                }
                Ok(XmlEvent::End(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "response" {
                        if in_response && !current_ics.trim().is_empty() {
                            events.push(RemoteEventData {
                                href: current_href.clone(),
                                etag: current_etag.clone(),
                                ics_content: current_ics.clone(),
                            });
                        }
                        in_response = false;
                    }
                    current_tag.clear();
                }
                Ok(XmlEvent::Eof) => break,
                Err(err) => return Err(anyhow!("XML parse error: {}", err)),
                _ => {}
            }
            buf.clear();
        }

        Ok(events)
    }

    /// Extract .ics resource URLs and ETags from Multi-Status PROPFIND response
    pub fn parse_multistatus_ics_resources(
        xml_text: &str,
        calendar_href: &str,
    ) -> Vec<(String, Option<String>)> {
        use quick_xml::events::Event as XmlEvent;
        use quick_xml::reader::Reader;

        let mut resources = Vec::new();
        let mut reader = Reader::from_str(xml_text);
        reader.config_mut().trim_text(true);

        let mut in_response = false;
        let mut current_href = String::new();
        let mut current_etag: Option<String> = None;
        let mut is_collection = false;
        let mut content_type = String::new();
        let mut current_tag = String::new();

        let mut buf = Vec::new();
        let norm_cal_href = calendar_href.trim_end_matches('/');

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(XmlEvent::Start(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "response" {
                        in_response = true;
                        current_href.clear();
                        current_etag = None;
                        is_collection = false;
                        content_type.clear();
                    } else if in_response {
                        if name_str == "collection" || name_str == "calendar" {
                            is_collection = true;
                        }
                        current_tag = name_str;
                    }
                }
                Ok(XmlEvent::Empty(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "collection" || name_str == "calendar" {
                        is_collection = true;
                    }
                }
                Ok(XmlEvent::Text(ref e)) => {
                    if in_response {
                        let text = e.unescape().unwrap_or_default().trim().to_string();
                        if current_tag == "href" && current_href.is_empty() {
                            current_href = text;
                        } else if current_tag == "getetag" {
                            current_etag = Some(text.trim_matches('"').to_string());
                        } else if current_tag == "getcontenttype" {
                            content_type = text;
                        }
                    }
                }
                Ok(XmlEvent::End(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "response" {
                        if in_response && !current_href.is_empty() && !is_collection {
                            let trimmed_href = current_href.trim_end_matches('/');
                            if trimmed_href != norm_cal_href {
                                if current_href.ends_with(".ics")
                                    || content_type.contains("calendar")
                                {
                                    resources.push((current_href.clone(), current_etag.clone()));
                                }
                            }
                        }
                        in_response = false;
                    }
                    current_tag.clear();
                }
                Ok(XmlEvent::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        resources
    }
}
