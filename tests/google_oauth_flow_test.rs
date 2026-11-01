use anyhow::Result;
use rcal::app::{AccountModalState, ActiveModal, App};
use rcal::mouse::HitTarget;
use rcal::storage::Database;
use rcal::sync::oauth::{listen_for_oauth_code, parse_query_param, OAuth2Config};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::test]
async fn test_oauth_config_for_google() {
    let config = OAuth2Config::for_google("custom-client-id", "custom-client-secret");
    assert_eq!(config.client_id, "custom-client-id");
    assert_eq!(config.client_secret, "custom-client-secret");
    assert_eq!(config.redirect_uri, "http://127.0.0.1:8088/oauth/callback");

    let auth_url = config.build_auth_url("state_abc_123");
    assert!(auth_url.contains("accounts.google.com/o/oauth2/v2/auth"));
    assert!(auth_url.contains("client_id=custom-client-id"));
    assert!(auth_url.contains("state=state_abc_123"));
    assert!(auth_url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A8088%2Foauth%2Fcallback"));
    assert!(auth_url.contains("response_type=code"));
}

#[tokio::test]
async fn test_oauth_config_for_microsoft() {
    let config = OAuth2Config::for_microsoft("custom-ms-id", "custom-ms-secret");
    assert_eq!(config.client_id, "custom-ms-id");
    assert_eq!(config.client_secret, "custom-ms-secret");
    assert_eq!(config.redirect_uri, "http://127.0.0.1:8088/oauth/callback");

    let auth_url = config.build_auth_url("ms_state_xyz");
    assert!(auth_url.contains("login.microsoftonline.com/common/oauth2/v2.0/authorize"));
    assert!(auth_url.contains("client_id=custom-ms-id"));
    assert!(auth_url.contains("state=ms_state_xyz"));
    assert!(auth_url.contains("Calendars.ReadWrite"));
    assert!(auth_url.contains("User.Read"));
}

#[test]
fn test_parse_query_param() {
    let req = "GET /oauth/callback?code=4%2F0AX4XfW&state=secure_state_xyz&scope=calendar HTTP/1.1";
    let code = parse_query_param(req, "code");
    assert_eq!(code, Some("4/0AX4XfW".to_string()));

    let state = parse_query_param(req, "state");
    assert_eq!(state, Some("secure_state_xyz".to_string()));

    let missing = parse_query_param(req, "non_existent");
    assert_eq!(missing, None);
}

use std::sync::atomic::{AtomicU16, Ordering};
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19200);

fn get_free_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[tokio::test]
async fn test_loopback_oauth_listener_success() -> Result<()> {
    let test_port = get_free_port();
    let expected_state = "test_rand_state_987";
    let test_code = "4/0AeanS093284092384";

    let server_handle = tokio::spawn(async move {
        listen_for_oauth_code(test_port, expected_state, 10, "Google Calendar").await
    });

    let mut client = None;
    for _ in 0..30 {
        if let Ok(s) = TcpStream::connect(format!("127.0.0.1:{}", test_port)).await {
            client = Some(s);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let mut client = client.expect("Failed to connect to listener");

    let http_request = format!(
        "GET /oauth/callback?code={}&state={} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nUser-Agent: TestBrowser\r\n\r\n",
        test_code, expected_state, test_port
    );
    client.write_all(http_request.as_bytes()).await?;

    let mut response_buf = [0u8; 2048];
    let n = client.read(&mut response_buf).await?;
    let response_str = String::from_utf8_lossy(&response_buf[..n]);

    assert!(response_str.starts_with("HTTP/1.1 200 OK"));
    assert!(response_str.contains("Google Calendar Connected"));

    let received_code = server_handle.await??;
    assert_eq!(received_code, test_code);

    Ok(())
}

#[tokio::test]
async fn test_loopback_oauth_listener_ms365_success() -> Result<()> {
    let test_port = get_free_port();
    let expected_state = "ms_test_state_555";
    let test_code = "M.R3_mock_ms_auth_code";

    let server_handle = tokio::spawn(async move {
        listen_for_oauth_code(test_port, expected_state, 10, "Microsoft 365").await
    });

    let mut client = None;
    for _ in 0..30 {
        if let Ok(s) = TcpStream::connect(format!("127.0.0.1:{}", test_port)).await {
            client = Some(s);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let mut client = client.expect("Failed to connect to listener");

    let http_request = format!(
        "GET /oauth/callback?code={}&state={} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        test_code, expected_state, test_port
    );
    client.write_all(http_request.as_bytes()).await?;

    let mut response_buf = [0u8; 2048];
    let n = client.read(&mut response_buf).await?;
    let response_str = String::from_utf8_lossy(&response_buf[..n]);

    assert!(response_str.starts_with("HTTP/1.1 200 OK"));
    assert!(response_str.contains("Microsoft 365 Connected"));

    let received_code = server_handle.await??;
    assert_eq!(received_code, test_code);

    Ok(())
}

#[tokio::test]
async fn test_loopback_oauth_listener_state_mismatch_rejection() -> Result<()> {
    let test_port = get_free_port();
    let expected_state = "correct_state_123";

    let server_handle = tokio::spawn(async move {
        listen_for_oauth_code(test_port, expected_state, 5, "Google Calendar").await
    });

    let mut client = None;
    for _ in 0..30 {
        if let Ok(s) = TcpStream::connect(format!("127.0.0.1:{}", test_port)).await {
            client = Some(s);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let mut client = client.expect("Failed to connect to listener");

    let http_request = format!(
        "GET /oauth/callback?code=some_code&state=wrong_state HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        test_port
    );
    client.write_all(http_request.as_bytes()).await?;

    let res = server_handle.await?;
    assert!(res.is_err(), "Listener must reject mismatched OAuth state (CSRF)");
    Ok(())
}

#[test]
fn test_google_preset_account_modal_interaction() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    let mut acc_state = AccountModalState::new();
    acc_state.is_adding = true;
    acc_state.set_preset(2); // Google Calendar preset

    assert_eq!(acc_state.provider_idx, 2);
    assert_eq!(acc_state.name, "Google Calendar");
    assert_eq!(acc_state.url, "https://www.googleapis.com/calendar/v3");

    app.modal = ActiveModal::Accounts(acc_state);

    // Register hit area and trigger OAuth login via handle_mouse_click
    app.hit_map.register(ratatui::layout::Rect::new(10, 10, 20, 2), HitTarget::ModalAccountOAuthLogin);
    let hit_res = app.handle_mouse_click(15, 11);
    assert!(hit_res.is_ok());

    if let ActiveModal::Accounts(ref state) = app.modal {
        assert!(state.is_oauth_waiting, "OAuth waiting state should be true after triggering login");
    } else {
        panic!("Modal should be Accounts");
    }

    Ok(())
}

#[test]
fn test_microsoft_preset_account_modal_interaction() -> Result<()> {
    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    let mut acc_state = AccountModalState::new();
    acc_state.is_adding = true;
    acc_state.set_preset(3); // Microsoft 365 preset

    assert_eq!(acc_state.provider_idx, 3);
    assert_eq!(acc_state.name, "Microsoft 365");
    assert_eq!(acc_state.url, "https://graph.microsoft.com/v1.0");

    app.modal = ActiveModal::Accounts(acc_state);

    // Register hit area and trigger OAuth login via handle_mouse_click
    app.hit_map.register(ratatui::layout::Rect::new(10, 10, 20, 2), HitTarget::ModalAccountOAuthLogin);
    let hit_res = app.handle_mouse_click(15, 11);
    assert!(hit_res.is_ok());

    if let ActiveModal::Accounts(ref state) = app.modal {
        assert!(state.is_oauth_waiting, "OAuth waiting state should be true after triggering MS login");
    } else {
        panic!("Modal should be Accounts");
    }

    Ok(())
}

#[tokio::test]
async fn test_google_one_click_placeholder_rejection() {
    use rcal::sync::oauth::execute_google_one_click_flow;

    // When no client ID or dummy placeholder is provided, execute_google_one_click_flow returns helpful error
    let res = execute_google_one_click_flow(None, None).await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Google Client ID가 필요합니다"));
    assert!(err_msg.contains("RCAL_GOOGLE_CLIENT_ID"));
}

#[test]
fn test_google_placeholder_detection() {
    assert!(OAuth2Config::is_placeholder_google_client(""));
    assert!(OAuth2Config::is_placeholder_google_client("   "));
    assert!(OAuth2Config::is_placeholder_google_client(OAuth2Config::GOOGLE_PLACEHOLDER_CLIENT_ID));
    assert!(!OAuth2Config::is_placeholder_google_client("1234567890-custom.apps.googleusercontent.com"));
}

#[test]
fn test_google_oauth_modal_error_box_rendering_and_wrapping() -> Result<()> {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use rcal::ui;

    let db = Database::in_memory()?;
    let mut app = App::new(db)?;

    let mut acc_state = AccountModalState::new();
    acc_state.is_adding = true;
    acc_state.set_preset(2); // Google Calendar preset
    acc_state.error_msg = Some(
        "Google Client ID가 필요합니다:\n• Google Cloud Console에서 '데스크톱 앱' OAuth Client ID 발급 후 입력\n• 또는 RCAL_GOOGLE_CLIENT_ID 환경변수를 설정해주세요.".to_string()
    );

    app.modal = ActiveModal::Accounts(acc_state);

    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        ui::render(f, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    let content_str: String = buffer.content().iter().map(|c| c.symbol()).collect();
    let clean_str: String = content_str.chars().filter(|c| !c.is_whitespace()).collect();

    // Verify error title and key error messages are present
    assert!(clean_str.contains("인증오류안내"), "Error title should be rendered inside modal");
    assert!(clean_str.contains("GoogleClientID가필요합니다"), "First line of error should be rendered");
    assert!(clean_str.contains("RCAL_GOOGLE_CLIENT_ID"), "Helpful env var guide should be rendered and wrapped");

    // Modal popup is centered in 100x30 with width 84 (popup_rect x = (100 - 84)/2 = 8, right border at x = 91)
    // Verify that columns x >= 93 outside the modal boundary do NOT contain error box content
    for y in 0..30 {
        for x in 92..100 {
            let symbol = buffer[(x, y)].symbol();
            assert!(!symbol.contains("Google") && !symbol.contains("Client") && !symbol.contains("RCAL"),
                "Text should not spill outside modal boundary at ({}, {})", x, y);
        }
    }

    Ok(())
}
