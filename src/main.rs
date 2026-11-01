use std::io::{self, stdout};
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use rcal::app::App;
use rcal::events;
use rcal::storage::Database;
use rcal::sync::{SyncEngine, SyncResult};
use rcal::ui;

#[tokio::main]
async fn main() -> Result<()> {
    // 0. Process command line arguments (e.g. --version, -v, --help)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" | "-V" => {
                println!("rcal {}", rcal::version());
                return Ok(());
            }
            "--help" | "-h" => {
                println!("rcal {}", rcal::version());
                println!("Fast, mouse-interactive TUI Calendar in Rust with multi-cloud sync\n");
                println!("Usage: rcal [OPTIONS]\n");
                println!("Options:");
                println!("  -v, --version  Print version information (CalVer: YYYY.MM.DD.N)");
                println!("  -h, --help     Print help information");
                return Ok(());
            }
            _ => {}
        }
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
    let db = Database::new(None)?;
    let mut app = App::new(db)?;

    // 4. Background Sync Worker Setup
    let (sync_trigger_tx, mut sync_trigger_rx) = mpsc::channel::<()>(5);
    let (sync_result_tx, mut sync_result_rx) = mpsc::channel::<SyncResult>(5);
    app.sync_tx = Some(sync_trigger_tx.clone());

    tokio::spawn(async move {
        // Auto-sync ticker (every 5 minutes)
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        // Worker DB instance
        let worker_db = match Database::new(None) {
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

    // 5. Main Event Loop
    let res = run_app(&mut terminal, &mut app, &mut sync_result_rx).await;

    // 6. Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
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
) -> Result<()> {
    while app.running {
        // Check for completed background syncs
        if let Ok(res) = sync_result_rx.try_recv() {
            app.is_syncing = false;
            let _ = app.reload_calendars();
            let _ = app.reload_events();

            if res.errors.is_empty() {
                if res.events_pulled > 0 || res.events_pushed > 0 {
                    app.set_status(format!("✓ Synced: +{} pushed, +{} pulled", res.events_pushed, res.events_pulled));
                } else if res.accounts_synced > 0 {
                    app.set_status("✓ All calendars up to date");
                }
            } else {
                app.set_status(format!("⚠ Sync note: {}", res.errors.first().unwrap_or(&"Error".to_string())));
            }
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
