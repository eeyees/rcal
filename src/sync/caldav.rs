use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, Method, StatusCode};
use url::Url;

#[derive(Debug, Clone)]
pub struct RemoteCalendarInfo {
    pub name: String,
    pub href: String,
    pub color_code: Option<String>,
    pub ctag: Option<String>,
}

#[derive(Debug, Clone)]
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
        let mut base_url = Url::parse(url_str)
            .with_context(|| format!("Invalid CalDAV URL: {}", url_str))?;

        // If path is empty, make sure it ends with /
        if base_url.path().is_empty() {
            base_url.set_path("/");
        }

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rcal/0.1.0 (Rust TUI Calendar)"));

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            base_url,
            username,
            password,
        })
    }

    /// Discover available calendars via PROPFIND
    pub async fn discover_calendars(&self) -> Result<Vec<RemoteCalendarInfo>> {
        let propfind_xml = r#"<?xml version="1.0" encoding="utf-8" ?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/">
  <d:prop>
    <d:displayname />
    <d:resourcetype />
    <a:calendar-color />
  </d:prop>
</d:propfind>"#;

        let res = self.client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), self.base_url.as_str())
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(propfind_xml)
            .send()
            .await
            .with_context(|| format!("Failed to connect to CalDAV server at {}", self.base_url))?;

        if !res.status().is_success() && res.status() != StatusCode::MULTI_STATUS {
            return Err(anyhow!("CalDAV PROPFIND failed with HTTP status: {}", res.status()));
        }

        let text = res.text().await?;
        Self::parse_multistatus_calendars(&self.base_url, &text)
    }

    /// Fetch events using a CalDAV calendar-query REPORT
    pub async fn fetch_events(&self, calendar_href: &str) -> Result<Vec<RemoteEventData>> {
        let calendar_url = self.resolve_url(calendar_href)?;

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

        let res = self.client
            .request(Method::from_bytes(b"REPORT").unwrap(), calendar_url.as_str())
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header(CONTENT_TYPE, "application/xml; charset=utf-8")
            .body(report_xml)
            .send()
            .await
            .with_context(|| format!("Failed to query events from {}", calendar_url))?;

        if !res.status().is_success() && res.status() != StatusCode::MULTI_STATUS {
            return Err(anyhow!("CalDAV REPORT failed with HTTP status: {}", res.status()));
        }

        let text = res.text().await?;
        Self::parse_multistatus_events(&text)
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

        let mut req = self.client
            .put(resolved_url.as_str())
            .basic_auth(&self.username, Some(&self.password))
            .header(CONTENT_TYPE, "text/calendar; charset=utf-8")
            .body(ics_content.to_string());

        if let Some(e) = etag {
            let quoted = if e.starts_with('"') { e.to_string() } else { format!("\"{}\"", e) };
            req = req.header("If-Match", quoted);
        }

        let res = req.send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("CalDAV PUT failed with HTTP status: {}", res.status()));
        }

        let new_etag = res.headers()
            .get("ETag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        Ok(new_etag)
    }

    /// Delete event via HTTP DELETE
    pub async fn delete_event(&self, event_href: &str, etag: Option<&str>) -> Result<()> {
        let resolved_url = self.resolve_url(event_href)?;

        let mut req = self.client
            .delete(resolved_url.as_str())
            .basic_auth(&self.username, Some(&self.password));

        if let Some(e) = etag {
            let quoted = if e.starts_with('"') { e.to_string() } else { format!("\"{}\"", e) };
            req = req.header("If-Match", quoted);
        }

        let res = req.send().await?;
        if !res.status().is_success() && res.status() != StatusCode::NOT_FOUND {
            return Err(anyhow!("CalDAV DELETE failed with HTTP status: {}", res.status()));
        }

        Ok(())
    }

    fn resolve_url(&self, href: &str) -> Result<Url> {
        if href.starts_with("http://") || href.starts_with("https://") {
            Url::parse(href).context("Invalid absolute URL")
        } else {
            self.base_url.join(href).context("Failed to join relative URL with base")
        }
    }

    fn parse_multistatus_calendars(base_url: &Url, xml_text: &str) -> Result<Vec<RemoteCalendarInfo>> {
        let mut calendars = Vec::new();
        // Simple XML parsing using quick-xml Reader
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
                        current_tag = name_str;
                    }
                }
                Ok(XmlEvent::Empty(ref e)) => {
                    let local = e.local_name();
                    let name_str = String::from_utf8_lossy(local.as_ref()).to_lowercase();
                    if name_str == "calendar" {
                        is_calendar = true;
                    }
                }
                Ok(XmlEvent::Text(ref e)) => {
                    if in_response {
                        let text = e.unescape().unwrap_or_default().to_string();
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

        // If no calendar with <calendar/> tag was discovered, try treating the base collection as one
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

    fn parse_multistatus_events(xml_text: &str) -> Result<Vec<RemoteEventData>> {
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
                            current_href = text;
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
                        if in_response && !current_ics.is_empty() {
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
}
