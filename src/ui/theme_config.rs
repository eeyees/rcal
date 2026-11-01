use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::ui::theme::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfigFile {
    #[serde(default)]
    pub active_theme: Option<String>,

    #[serde(default)]
    pub options: VisibilityOptions,

    #[serde(default)]
    pub themes: HashMap<String, ThemeColorOverrides>,

    #[serde(default)]
    pub custom_themes: Vec<CustomThemeDef>,
}

impl Default for ThemeConfigFile {
    fn default() -> Self {
        Self {
            active_theme: Some("modern_dark".to_string()),
            options: VisibilityOptions::default(),
            themes: HashMap::new(),
            custom_themes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisibilityOptions {
    /// If true, forces opaque backgrounds for all themes (avoids transparency contrast issues).
    #[serde(default)]
    pub force_opaque_bg: bool,

    /// High contrast mode: sharpens borders and boosts foreground text contrast.
    #[serde(default)]
    pub high_contrast: bool,

    /// Boost muted text: brightens secondary dates, shortcut hints, and dimmed indicators.
    #[serde(default)]
    pub boost_muted_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeColorOverrides {
    pub bg: Option<String>,
    pub panel_bg: Option<String>,
    pub border: Option<String>,
    pub border_focused: Option<String>,
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub accent: Option<String>,
    pub danger: Option<String>,
    pub text_normal: Option<String>,
    pub text_muted: Option<String>,
    pub text_dim: Option<String>,
    pub text_highlight: Option<String>,
    pub today_bg: Option<String>,
    pub selected_bg: Option<String>,
    pub tab_active_bg: Option<String>,
    pub tab_inactive_bg: Option<String>,
    pub button_bg: Option<String>,
    pub button_fg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomThemeDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_light: bool,
    pub bg: Option<String>,
    pub panel_bg: String,
    pub border: String,
    pub border_focused: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub danger: String,
    pub text_normal: String,
    pub text_muted: String,
    pub text_dim: String,
    pub text_highlight: String,
    pub today_bg: String,
    pub selected_bg: String,
    pub tab_active_bg: String,
    pub tab_inactive_bg: String,
    pub button_bg: String,
    pub button_fg: String,
}

impl ThemeConfigFile {
    /// Determines the standard config file path:
    /// 1. $RCAL_CONFIG_DIR/theme.toml (if env var set)
    /// 2. ~/.config/rcal/theme.toml (if exists or on Linux/macOS)
    /// 3. dirs::config_dir()/rcal/theme.toml
    /// 4. ./theme.toml fallback
    pub fn config_path() -> PathBuf {
        if let Ok(dir) = std::env::var("RCAL_CONFIG_DIR") {
            return PathBuf::from(dir).join("theme.toml");
        }

        if let Some(home) = dirs::home_dir() {
            let dot_config = home.join(".config").join("rcal").join("theme.toml");
            if dot_config.exists() {
                return dot_config;
            }
        }

        if let Some(config_dir) = dirs::config_dir() {
            return config_dir.join("rcal").join("theme.toml");
        }

        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("rcal").join("theme.toml");
        }

        PathBuf::from("theme.toml")
    }

    /// Loads the config file from disk. If the file does not exist, creates parent dirs
    /// and writes a heavily documented template file with helpful comments and examples.
    pub fn load_or_create() -> (Self, PathBuf) {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str::<ThemeConfigFile>(&content) {
                    Ok(cfg) => return (cfg, path),
                    Err(e) => {
                        eprintln!("[rcal] Warning: failed to parse {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("[rcal] Warning: failed to read {}: {}", path.display(), e);
                }
            }
        }

        // Generate template file
        let template = Self::generate_annotated_template();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, template);

        (Self::default(), path)
    }

    /// Generates a well-annotated default theme configuration template.
    pub fn generate_annotated_template() -> String {
        r##"# ==============================================================================
#  rcal Color Theme & Visibility Configuration
#  Path: ~/.config/rcal/theme.toml
# ==============================================================================
# This file allows you to customize and fine-tune any color in rcal.
# If certain text, borders, or highlights are hard to read on your terminal emulator
# or monitor, you can override specific colors or enable contrast enhancements below.

# Active theme name.
# You can also change themes dynamically inside rcal via shortcut [T] or [Settings -> Theme].
# Available themes (37 total):
#   - Dark:
#       modern_dark, nord, dracula, tokyo_night, catppuccin_mocha,
#       catppuccin_macchiato, one_dark, github_dark, night_owl, cobalt2
#   - Nature / Earthy:
#       forest, everforest_dark, kanagawa, rose_pine, rose_pine_moon,
#       palenight, gruvbox_dark, ayu_dark, catppuccin_frappe
#   - Vibrant / Retro:
#       sunset, monokai, cyberpunk, synthwave84, matrix,
#       ayu_mirage, tokyo_night_storm, solarized_dark
#   - Light Themes:
#       solarized_light, catppuccin_latte, tokyo_night_light, gruvbox_light,
#       one_light, github_light, rose_pine_dawn, everforest_light,
#       light_owl, ayu_light
active_theme = "modern_dark"

[options]
# Set to true to force opaque background color (useful if terminal transparency hurts contrast)
force_opaque_bg = false

# Set to true to boost border sharpness and general text contrast
high_contrast = false

# Set to true to make muted dates, dimmed hours, and shortcut hints brighter and easier to read
boost_muted_text = false

# ------------------------------------------------------------------------------
# Per-Theme Custom Color Overrides
# ------------------------------------------------------------------------------
# You can override any individual color for any theme.
# Colors can be specified as Hex ("#RRGGBB", "#RGB") or RGB ("rgb(r, g, b)").
# Available properties:
#   bg, panel_bg, border, border_focused,
#   primary, secondary, accent, danger,
#   text_normal, text_muted, text_dim, text_highlight,
#   today_bg, selected_bg, tab_active_bg, tab_inactive_bg,
#   button_bg, button_fg

[themes.tokyo_night]
# border = "#7aa2f7"
# text_muted = "#9aa5ce"
# today_bg = "#3d59a1"

[themes.github_light]
# text_normal = "#1f2328"
# border = "#d0d7de"
# selected_bg = "#eaeef2"

[themes.gruvbox_dark]
# border_focused = "#fabd2f"
# text_muted = "#bdae93"

# ------------------------------------------------------------------------------
# User-Defined Custom Themes (Optional)
# ------------------------------------------------------------------------------
# [[custom_themes]]
# id = "custom_cyber_gold"
# name = "Cyber Gold"
# is_light = false
# panel_bg = "#121218"
# border = "#403d52"
# border_focused = "#f6c177"
# primary = "#f6c177"
# secondary = "#9ccfd8"
# accent = "#eb6f92"
# danger = "#eb6f92"
# text_normal = "#e0def4"
# text_muted = "#908caa"
# text_dim = "#6e6a86"
# text_highlight = "#ffffff"
# today_bg = "#433822"
# selected_bg = "#2a283e"
# tab_active_bg = "#c4a7e7"
# tab_inactive_bg = "#1f1d2e"
# button_bg = "#2a283e"
# button_fg = "#e0def4"
"##
        .to_string()
    }

    /// Applies configuration overrides, high-contrast, and visibility tweaks to a `Theme`.
    pub fn apply_to(&self, theme: &mut Theme) {
        let key = theme.mode.as_str();

        // 1. Apply per-theme overrides if present
        if let Some(ov) = self.themes.get(key) {
            apply_overrides(theme, ov);
        }

        // Also check with hyphens vs underscores
        let alt_key = key.replace('_', "-");
        if alt_key != key {
            if let Some(ov) = self.themes.get(&alt_key) {
                apply_overrides(theme, ov);
            }
        }

        // 2. Visibility options
        let is_light = theme.mode.is_light();

        if self.options.force_opaque_bg && theme.bg == Color::Reset {
            theme.bg = theme.panel_bg;
        }

        if self.options.boost_muted_text {
            if is_light {
                // In light themes, make muted text darker for better readability
                theme.text_muted = Color::Rgb(60, 65, 75);
                theme.text_dim = Color::Rgb(100, 105, 120);
            } else {
                // In dark themes, brighten muted text
                theme.text_muted = Color::Rgb(190, 195, 210);
                theme.text_dim = Color::Rgb(130, 135, 150);
            }
        }

        if self.options.high_contrast {
            if is_light {
                theme.border = Color::Rgb(100, 105, 115);
                theme.text_normal = Color::Rgb(0, 0, 0);
                theme.text_muted = Color::Rgb(40, 45, 55);
                theme.text_dim = Color::Rgb(80, 85, 95);
            } else {
                theme.border = Color::Rgb(110, 115, 135);
                theme.text_normal = Color::Rgb(255, 255, 255);
                theme.text_muted = Color::Rgb(210, 215, 230);
                theme.text_dim = Color::Rgb(150, 155, 170);
            }
        }
    }
}

fn apply_overrides(theme: &mut Theme, ov: &ThemeColorOverrides) {
    if let Some(ref s) = ov.bg {
        if let Some(c) = parse_color(s) {
            theme.bg = c;
        }
    }
    if let Some(ref s) = ov.panel_bg {
        if let Some(c) = parse_color(s) {
            theme.panel_bg = c;
        }
    }
    if let Some(ref s) = ov.border {
        if let Some(c) = parse_color(s) {
            theme.border = c;
        }
    }
    if let Some(ref s) = ov.border_focused {
        if let Some(c) = parse_color(s) {
            theme.border_focused = c;
        }
    }
    if let Some(ref s) = ov.primary {
        if let Some(c) = parse_color(s) {
            theme.primary = c;
        }
    }
    if let Some(ref s) = ov.secondary {
        if let Some(c) = parse_color(s) {
            theme.secondary = c;
        }
    }
    if let Some(ref s) = ov.accent {
        if let Some(c) = parse_color(s) {
            theme.accent = c;
        }
    }
    if let Some(ref s) = ov.danger {
        if let Some(c) = parse_color(s) {
            theme.danger = c;
        }
    }
    if let Some(ref s) = ov.text_normal {
        if let Some(c) = parse_color(s) {
            theme.text_normal = c;
        }
    }
    if let Some(ref s) = ov.text_muted {
        if let Some(c) = parse_color(s) {
            theme.text_muted = c;
        }
    }
    if let Some(ref s) = ov.text_dim {
        if let Some(c) = parse_color(s) {
            theme.text_dim = c;
        }
    }
    if let Some(ref s) = ov.text_highlight {
        if let Some(c) = parse_color(s) {
            theme.text_highlight = c;
        }
    }
    if let Some(ref s) = ov.today_bg {
        if let Some(c) = parse_color(s) {
            theme.today_bg = c;
        }
    }
    if let Some(ref s) = ov.selected_bg {
        if let Some(c) = parse_color(s) {
            theme.selected_bg = c;
        }
    }
    if let Some(ref s) = ov.tab_active_bg {
        if let Some(c) = parse_color(s) {
            theme.tab_active_bg = c;
        }
    }
    if let Some(ref s) = ov.tab_inactive_bg {
        if let Some(c) = parse_color(s) {
            theme.tab_inactive_bg = c;
        }
    }
    if let Some(ref s) = ov.button_bg {
        if let Some(c) = parse_color(s) {
            theme.button_bg = c;
        }
    }
    if let Some(ref s) = ov.button_fg {
        if let Some(c) = parse_color(s) {
            theme.button_fg = c;
        }
    }
}

/// Parses a color string into Ratatui `Color`.
/// Supports:
/// - Hex: `"#FFFFFF"`, `"#fff"`
/// - Functional: `"rgb(255, 255, 255)"`
/// - Named: `"reset"`, `"black"`, `"white"`, `"red"`, `"green"`, `"blue"`, `"cyan"`, `"yellow"`, `"magenta"`
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s.eq_ignore_ascii_case("reset") || s.eq_ignore_ascii_case("transparent") {
        return Some(Color::Reset);
    }
    if s.eq_ignore_ascii_case("black") {
        return Some(Color::Black);
    }
    if s.eq_ignore_ascii_case("white") {
        return Some(Color::White);
    }
    if s.eq_ignore_ascii_case("red") {
        return Some(Color::Red);
    }
    if s.eq_ignore_ascii_case("green") {
        return Some(Color::Green);
    }
    if s.eq_ignore_ascii_case("yellow") {
        return Some(Color::Yellow);
    }
    if s.eq_ignore_ascii_case("blue") {
        return Some(Color::Blue);
    }
    if s.eq_ignore_ascii_case("magenta") {
        return Some(Color::Magenta);
    }
    if s.eq_ignore_ascii_case("cyan") {
        return Some(Color::Cyan);
    }

    // Hex parsing
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(Color::Rgb(r, g, b));
        }
    }

    // rgb(r, g, b) parsing
    if s.to_lowercase().starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }

    None
}
