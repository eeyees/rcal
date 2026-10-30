use std::io::{self, stdout};
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use rcal::app::App;
use rcal::events;
use rcal::storage::Database;
use rcal::ui;

#[tokio::main]
async fn main() -> Result<()> {
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

    // 4. Main Event Loop
    let res = run_app(&mut terminal, &mut app).await;

    // 5. Restore terminal
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
) -> Result<()> {
    while app.running {
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
