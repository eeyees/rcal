use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::time::Duration;
use tokio::sync::mpsc;

use rcal::app::App;
use rcal::events;
use rcal::notification::{check_and_notify_upcoming, NotificationConfig};
use rcal::storage::Database;
use rcal::sync::{SyncEngine, SyncResult};
use rcal::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // 0. Process command line arguments (e.g. --version, -v, --help, mcp)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        for arg in &args[1..] {
            match arg.as_str() {
                "--version" | "-v" | "-V" => {
                    println!("rcal {}", rcal::version());
                    return Ok(());
                }
                "--help" | "-h" => {
                    println!("rcal {}", rcal::version());
                    println!(
                        "Fast, mouse-interactive TUI Calendar in Rust with multi-cloud sync\n"
                    );
                    println!("Usage: rcal [OPTIONS] [COMMAND]\n");
                    println!("Commands:");
                    println!("  notify                 Check upcoming events and trigger notifications once");
                    println!(
                        "  notify --daemon        Run notification checker daemon in background"
                    );
                    println!("  notify --test          Send a test desktop notification");
                    println!("  mcp, --mcp             Run Model Context Protocol (MCP) server over stdio");
                    println!();
                    println!("Options:");
                    println!("  --db <PATH>            Path to SQLite database file");
                    println!(
                        "  -v, --version          Print version information (CalVer: YYYY.MM.DD.N)"
                    );
                    println!("  -h, --help             Print help information");
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    let custom_db = parse_db_flag(&args);

    // MCP server mode for AI agents
    if args
        .iter()
        .any(|a| a == "mcp" || a == "--mcp" || a == "mcp-server")
    {
        let db = Database::new(custom_db.as_deref())?;
        rcal::mcp::run_stdio(db).await?;
        return Ok(());
    }

    // CLI Notification commands
    if args.len() > 1 && (args[1] == "notify" || args[1] == "--notify") {
        let db = Database::new(custom_db.as_deref())?;
        let config = NotificationConfig::load_from_db(&db);
        let lang = db
            .get_setting("language")
            .ok()
            .flatten()
            .and_then(|s| rcal::i18n::Language::from_code(&s))
            .unwrap_or_else(rcal::i18n::Language::detect);

        if args.iter().any(|a| a == "--test" || a == "-t") {
            println!("Sending test desktop notification...");
            rcal::notification::send_test_notification(lang, config.sound)?;
            println!("✓ Test desktop notification sent.");
            return Ok(());
        }

        if args.iter().any(|a| a == "--daemon" || a == "-d") {
            println!(
                "rcal notification daemon started. Checking every 15 seconds (lead time: {}m). Press Ctrl+C to stop.",
                config.lead_minutes
            );
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            loop {
                interval.tick().await;
                let cfg = NotificationConfig::load_from_db(&db);
                if cfg.enabled {
                    if let Ok(notified) =
                        check_and_notify_upcoming(&db, &cfg, chrono::Local::now(), lang)
                    {
                        for (ev, cal) in notified {
                            println!(
                                "[{}] Notified: '{}' [{}]",
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                                ev.title,
                                cal
                            );
                        }
                    }
                }
            }
        }

        // One-shot check
        let notified = check_and_notify_upcoming(&db, &config, chrono::Local::now(), lang)?;
        if notified.is_empty() {
            println!("No upcoming events requiring notification at this time.");
        } else {
            for (ev, cal) in notified {
                println!("✓ Notified: '{}' [{}]", ev.title, cal);
            }
        }
        return Ok(());
    }

    // 1. Setup panic hook to guarantee terminal cleanup on crash
    setup_panic_hook();

    // 2. Initialize terminal with alternate screen and mouse capture
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Initialize storage and application state
    let db = Database::new(custom_db.as_deref())?;
    let mut app = App::new(db)?;

    // 4. Background Sync Worker Setup
    let (sync_trigger_tx, mut sync_trigger_rx) = mpsc::channel::<()>(5);
    let (sync_result_tx, mut sync_result_rx) = mpsc::channel::<SyncResult>(5);
    app.sync_tx = Some(sync_trigger_tx.clone());

    let worker_db_path = custom_db.clone();
    tokio::spawn(async move {
        // Auto-sync ticker (every 5 minutes)
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        // Worker DB instance
        let worker_db = match Database::new(worker_db_path.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to open worker db: {:?}", e);
                return;
            }
        };

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let res = SyncEngine::sync_all(&worker_db).await;
                    let _ = sync_result_tx.send(res).await;
                }
                Some(_) = sync_trigger_rx.recv() => {
                    let res = SyncEngine::sync_all(&worker_db).await;
                    let _ = sync_result_tx.send(res).await;
                }
            }
        }
    });

    // 5. Background Notification Worker Setup
    let (notif_tx, mut notif_rx) = mpsc::channel::<String>(10);
    let notif_db_path = custom_db.clone();
    tokio::spawn(async move {
        // Notification check interval: every 15 seconds
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        let worker_db = match Database::new(notif_db_path.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to open worker db for notifications: {:?}", e);
                return;
            }
        };

        let _ =
            worker_db.cleanup_old_notifications(chrono::Local::now() - chrono::Duration::days(7));

        loop {
            interval.tick().await;

            let config = NotificationConfig::load_from_db(&worker_db);
            if !config.enabled {
                continue;
            }

            let lang = worker_db
                .get_setting("language")
                .ok()
                .flatten()
                .and_then(|s| rcal::i18n::Language::from_code(&s))
                .unwrap_or_else(rcal::i18n::Language::detect);

            if let Ok(notified) =
                check_and_notify_upcoming(&worker_db, &config, chrono::Local::now(), lang)
            {
                for (ev, _) in notified {
                    let _ = notif_tx.send(ev.title).await;
                }
            }
        }
    });

    // 6. OAuth Result Channel Setup
    let (oauth_result_tx, mut oauth_result_rx) =
        mpsc::channel::<Result<rcal::model::CalDavAccount, String>>(2);
    app.oauth_result_tx = Some(oauth_result_tx);

    // 7. Main Event Loop
    let res = run_app(
        &mut terminal,
        &mut app,
        &mut sync_result_rx,
        &mut notif_rx,
        &mut oauth_result_rx,
    )
    .await;

    // 8. Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running rcal: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    sync_result_rx: &mut mpsc::Receiver<SyncResult>,
    notif_rx: &mut mpsc::Receiver<String>,
    oauth_result_rx: &mut mpsc::Receiver<Result<rcal::model::CalDavAccount, String>>,
) -> Result<()> {
    while app.running {
        // Check for completed background syncs
        if let Ok(res) = sync_result_rx.try_recv() {
            app.is_syncing = false;
            let _ = app.reload_calendars();
            let _ = app.reload_events();

            if res.errors.is_empty() {
                if res.events_pulled > 0 || res.events_pushed > 0 {
                    app.set_status(format!(
                        "✓  Synced: +{} pushed, +{} pulled",
                        res.events_pushed, res.events_pulled
                    ));
                } else if res.accounts_synced > 0 {
                    app.set_status("✓  All calendars up to date");
                }
            } else {
                app.set_status(format!(
                    "⚠  Sync note: {}",
                    res.errors.first().unwrap_or(&"Error".to_string())
                ));
            }
        }

        // Check for completed OAuth logins
        if let Ok(oauth_res) = oauth_result_rx.try_recv() {
            match oauth_res {
                Ok(new_acc) => {
                    let _ = app.db.save_account(&new_acc);
                    let _ = app.reload_calendars();
                    let _ = app.reload_events();
                    let msg = match app.lang {
                        rcal::i18n::Language::Ko => {
                            format!("✓  {} 계정 연동 성공! 동기화 중...", new_acc.name)
                        }
                        rcal::i18n::Language::Ja => {
                            format!("✓  {} 連携完了! 同期中...", new_acc.name)
                        }
                        _ => format!("✓  {} connected! Syncing...", new_acc.name),
                    };
                    app.set_status(msg);
                    app.trigger_sync();
                    app.close_modal();
                }
                Err(err) => {
                    app.set_status(format!("⚠  OAuth: {}", err));
                    if let rcal::app::ActiveModal::Accounts(ref mut acc_state) = app.modal {
                        acc_state.is_oauth_waiting = false;
                        acc_state.error_msg = Some(err);
                    }
                }
            }
        }

        // Check for notifications triggered by background worker
        if let Ok(title) = notif_rx.try_recv() {
            let msg = match app.lang {
                rcal::i18n::Language::Ko => format!("🔔  알림 발송: {}", title),
                rcal::i18n::Language::Ja => format!("🔔  通知送信: {}", title),
                _ => format!("🔔  Notification sent: {}", title),
            };
            app.set_status(msg);
        }

        terminal.draw(|f| ui::render(f, app))?;

        // 50ms poll gives responsive 20fps tick for clock updates & smooth responsiveness
        if event::poll(Duration::from_millis(50))? {
            let evt = event::read()?;
            events::handle_crossterm_event(app, evt)?;
        }
    }
    Ok(())
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

fn parse_db_flag(args: &[String]) -> Option<std::path::PathBuf> {
    for i in 0..args.len() {
        if (args[i] == "--db" || args[i] == "--database") && i + 1 < args.len() {
            return Some(std::path::PathBuf::from(&args[i + 1]));
        }
    }
    None
}
