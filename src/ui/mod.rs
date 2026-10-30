pub mod agenda_view;
pub mod day_view;
pub mod footer;
pub mod header;
pub mod modals;
pub mod month_view;
pub mod sidebar;
pub mod theme;
pub mod week_view;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{ActiveModal, App, ViewMode};

pub fn render(f: &mut Frame, app: &mut App) {
    // Clear hit map before rendering this frame
    app.hit_map.clear();

    let size = f.area();

    // Top Header (3), Main Body (Min), Footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Spacer
            Constraint::Min(15),   // Main Workspace (Sidebar + Calendar View)
            Constraint::Length(1), // Footer status bar
        ])
        .split(size);

    // 1. Header
    header::render(f, app, chunks[0]);

    // 2. Main Workspace (Sidebar + View)
    let main_workspace = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26), // Sidebar: Calendars + tips
            Constraint::Min(50),    // Active View
        ])
        .split(chunks[2]);

    sidebar::render(f, app, main_workspace[0]);

    let view_area = main_workspace[1];
    match app.current_view {
        ViewMode::Month => month_view::render(f, app, view_area),
        ViewMode::Week => week_view::render(f, app, view_area),
        ViewMode::Day => day_view::render(f, app, view_area),
        ViewMode::Agenda => agenda_view::render(f, app, view_area),
    }

    // 3. Footer
    footer::render(f, app, chunks[3]);

    // 4. Modals (Overlays on top)
    match &app.modal {
        ActiveModal::EventForm(form) => {
            let form_clone = form.clone();
            modals::event_form::render(f, app, &form_clone, size);
        }
        ActiveModal::EventDetail(event_id) => {
            let id_clone = event_id.clone();
            modals::event_detail::render(f, app, &id_clone, size);
        }
        ActiveModal::Help => {
            modals::help::render(f, app, size);
        }
        ActiveModal::ConfirmDelete(_) => {
            // Can be extended or handled via EventDetail
        }
        ActiveModal::None => {}
    }
}
