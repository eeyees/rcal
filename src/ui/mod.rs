pub mod agenda_view;
pub mod day_view;
pub mod footer;
pub mod header;
pub mod modals;
pub mod month_view;
pub mod sidebar;
pub mod theme;
pub mod theme_config;
pub mod week_view;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{ActiveModal, App, ViewMode};
use crate::mouse::HitTarget;

pub fn render(f: &mut Frame, app: &mut App) {
    // Clear hit map before rendering this frame
    app.hit_map.clear();

    let size = f.area();

    // Fill background if theme defines explicit color (e.g., light themes or opaque background mode)
    if app.theme.bg != ratatui::style::Color::Reset {
        f.render_widget(
            ratatui::widgets::Block::default()
                .style(ratatui::style::Style::default().bg(app.theme.bg)),
            size,
        );
    }

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
    let view_area = if app.show_sidebar {
        let sidebar_width = if size.width >= 120 {
            34
        } else if size.width >= 100 {
            32
        } else {
            28
        };

        let main_workspace = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width), // Adaptive sidebar width to prevent any clipping
                Constraint::Min(50),               // Active View
            ])
            .split(chunks[2]);

        sidebar::render(f, app, main_workspace[0]);
        main_workspace[1]
    } else {
        // When sidebar is hidden, active view gets the full width (100% of chunks[2])
        chunks[2]
    };

    match app.current_view {
        ViewMode::Month => month_view::render(f, app, view_area),
        ViewMode::Week => week_view::render(f, app, view_area),
        ViewMode::Day => day_view::render(f, app, view_area),
        ViewMode::Agenda => agenda_view::render(f, app, view_area),
    }

    // 3. Footer
    footer::render(f, app, chunks[3]);

    // 4. Modals (Overlays on top)
    if !matches!(app.modal, ActiveModal::None) {
        // Register full-screen backdrop so mouse clicks outside dialog cannot click background
        app.hit_map.register(size, HitTarget::ModalBackdrop);
    }

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
        ActiveModal::Accounts(ref state) => {
            let state_clone = state.clone();
            modals::account_modal::render(f, app, &state_clone, size);
        }
        ActiveModal::CalendarManager(ref state) => {
            let state_clone = state.clone();
            modals::calendar_modal::render(f, app, &state_clone, size);
        }
        ActiveModal::Settings => {
            modals::settings_modal::render(f, app, size);
        }
        ActiveModal::Language(ref state) => {
            let state_clone = state.clone();
            modals::language_modal::render(f, app, &state_clone, size);
        }
        ActiveModal::Theme(ref state) => {
            let state_clone = state.clone();
            modals::theme_modal::render(f, app, &state_clone, size);
        }
        ActiveModal::ConfirmDelete(ref target) => {
            let target_clone = target.clone();
            modals::confirm_modal::render(f, app, &target_clone, size);
        }
        ActiveModal::None => {}
    }
}
