use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // Primary layout colors
    pub const BG: Color = Color::Reset;
    pub const PANEL_BG: Color = Color::Rgb(22, 24, 30);
    pub const BORDER: Color = Color::Rgb(60, 65, 80);
    pub const BORDER_FOCUSED: Color = Color::Rgb(100, 160, 255);

    // Accent colors
    pub const PRIMARY: Color = Color::Rgb(80, 160, 255);   // Modern Bright Blue
    pub const SECONDARY: Color = Color::Rgb(70, 205, 140); // Soft Emerald Green
    pub const ACCENT: Color = Color::Rgb(250, 190, 80);    // Warm Amber
    pub const DANGER: Color = Color::Rgb(255, 90, 90);     // Coral Red

    // Text colors
    pub const TEXT_NORMAL: Color = Color::Rgb(220, 225, 235);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 135, 150);
    pub const TEXT_DIM: Color = Color::Rgb(80, 85, 100);
    pub const TEXT_HIGHLIGHT: Color = Color::Rgb(255, 255, 255);

    // Day styles
    pub fn today_style() -> Style {
        Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(30, 90, 180))
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_day_style() -> Style {
        Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(60, 75, 110))
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_tab_active() -> Style {
        Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(40, 100, 210))
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_tab_inactive() -> Style {
        Style::default()
            .fg(Color::Rgb(160, 170, 190))
            .bg(Color::Rgb(35, 40, 50))
    }

    pub fn button_style() -> Style {
        Style::default()
            .fg(Color::Rgb(240, 240, 250))
            .bg(Color::Rgb(50, 58, 75))
            .add_modifier(Modifier::BOLD)
    }
}
