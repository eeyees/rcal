use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    ModernDark,
    Nord,
    Dracula,
    Forest,
    Sunset,
    Monokai,
    Solarized,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::ModernDark => "modern_dark",
            ThemeMode::Nord => "nord",
            ThemeMode::Dracula => "dracula",
            ThemeMode::Forest => "forest",
            ThemeMode::Sunset => "sunset",
            ThemeMode::Monokai => "monokai",
            ThemeMode::Solarized => "solarized",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().replace('-', "_").as_str() {
            "nord" => ThemeMode::Nord,
            "dracula" => ThemeMode::Dracula,
            "forest" | "matcha" => ThemeMode::Forest,
            "sunset" | "cyberpunk" => ThemeMode::Sunset,
            "monokai" => ThemeMode::Monokai,
            "solarized" | "solarized_dark" => ThemeMode::Solarized,
            _ => ThemeMode::ModernDark,
        }
    }

    pub fn all() -> &'static [ThemeMode] {
        &[
            ThemeMode::ModernDark,
            ThemeMode::Nord,
            ThemeMode::Dracula,
            ThemeMode::Forest,
            ThemeMode::Sunset,
            ThemeMode::Monokai,
            ThemeMode::Solarized,
        ]
    }

    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|m| m == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|m| m == self).unwrap_or(0);
        if idx == 0 {
            all[all.len() - 1]
        } else {
            all[idx - 1]
        }
    }

    pub fn label(&self, lang: crate::i18n::Language) -> &'static str {
        match self {
            ThemeMode::ModernDark => match lang {
                crate::i18n::Language::Ko => "모던 다크 (Modern Dark)",
                crate::i18n::Language::Ja => "モダンダーク (Modern Dark)",
                _ => "Modern Dark",
            },
            ThemeMode::Nord => match lang {
                crate::i18n::Language::Ko => "노르딕 (Nord)",
                crate::i18n::Language::Ja => "ノルディック (Nord)",
                _ => "Nord",
            },
            ThemeMode::Dracula => match lang {
                crate::i18n::Language::Ko => "드라큘라 (Dracula)",
                crate::i18n::Language::Ja => "ドラキュラ (Dracula)",
                _ => "Dracula",
            },
            ThemeMode::Forest => match lang {
                crate::i18n::Language::Ko => "포레스트 (Forest)",
                crate::i18n::Language::Ja => "フォレスト (Forest)",
                _ => "Forest",
            },
            ThemeMode::Sunset => match lang {
                crate::i18n::Language::Ko => "선셋 (Sunset)",
                crate::i18n::Language::Ja => "サンセット (Sunset)",
                _ => "Sunset",
            },
            ThemeMode::Monokai => match lang {
                crate::i18n::Language::Ko => "모노카이 (Monokai)",
                crate::i18n::Language::Ja => "モノカイ (Monokai)",
                _ => "Monokai",
            },
            ThemeMode::Solarized => match lang {
                crate::i18n::Language::Ko => "솔라라이즈드 (Solarized)",
                crate::i18n::Language::Ja => "ソーラライズド (Solarized)",
                _ => "Solarized",
            },
        }
    }

    pub fn short_name(&self, lang: crate::i18n::Language) -> &'static str {
        match self {
            ThemeMode::ModernDark => match lang {
                crate::i18n::Language::Ko => "모던다크",
                crate::i18n::Language::Ja => "モダン",
                _ => "Modern",
            },
            ThemeMode::Nord => match lang {
                crate::i18n::Language::Ko => "노르딕",
                crate::i18n::Language::Ja => "ノルディック",
                _ => "Nord",
            },
            ThemeMode::Dracula => match lang {
                crate::i18n::Language::Ko => "드라큘라",
                crate::i18n::Language::Ja => "ドラキュラ",
                _ => "Dracula",
            },
            ThemeMode::Forest => match lang {
                crate::i18n::Language::Ko => "포레스트",
                crate::i18n::Language::Ja => "フォレスト",
                _ => "Forest",
            },
            ThemeMode::Sunset => match lang {
                crate::i18n::Language::Ko => "선셋",
                crate::i18n::Language::Ja => "サンセット",
                _ => "Sunset",
            },
            ThemeMode::Monokai => match lang {
                crate::i18n::Language::Ko => "모노카이",
                crate::i18n::Language::Ja => "モノカイ",
                _ => "Monokai",
            },
            ThemeMode::Solarized => match lang {
                crate::i18n::Language::Ko => "솔라라이즈드",
                crate::i18n::Language::Ja => "ソーラー",
                _ => "Solarized",
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub mode: ThemeMode,

    // Primary layout colors
    pub bg: Color,
    pub panel_bg: Color,
    pub border: Color,
    pub border_focused: Color,

    // Accent colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub danger: Color,

    // Text colors
    pub text_normal: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub text_highlight: Color,

    // Specific component colors
    pub today_bg: Color,
    pub selected_bg: Color,
    pub tab_active_bg: Color,
    pub tab_inactive_bg: Color,
    pub button_bg: Color,
    pub button_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_mode(ThemeMode::ModernDark)
    }
}

#[allow(dead_code)]
impl Theme {
    // Static fallback constants (Modern Dark) for backwards compatibility
    pub const BG: Color = Color::Reset;
    pub const PANEL_BG: Color = Color::Rgb(22, 24, 30);
    pub const BORDER: Color = Color::Rgb(60, 65, 80);
    pub const BORDER_FOCUSED: Color = Color::Rgb(100, 160, 255);

    pub const PRIMARY: Color = Color::Rgb(80, 160, 255);   // Modern Bright Blue
    pub const SECONDARY: Color = Color::Rgb(70, 205, 140); // Soft Emerald Green
    pub const ACCENT: Color = Color::Rgb(250, 190, 80);    // Warm Amber
    pub const DANGER: Color = Color::Rgb(255, 90, 90);     // Coral Red

    pub const TEXT_NORMAL: Color = Color::Rgb(220, 225, 235);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 135, 150);
    pub const TEXT_DIM: Color = Color::Rgb(80, 85, 100);
    pub const TEXT_HIGHLIGHT: Color = Color::Rgb(255, 255, 255);

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::ModernDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(22, 24, 30),
                border: Color::Rgb(60, 65, 80),
                border_focused: Color::Rgb(100, 160, 255),
                primary: Color::Rgb(80, 160, 255),
                secondary: Color::Rgb(70, 205, 140),
                accent: Color::Rgb(250, 190, 80),
                danger: Color::Rgb(255, 90, 90),
                text_normal: Color::Rgb(220, 225, 235),
                text_muted: Color::Rgb(130, 135, 150),
                text_dim: Color::Rgb(80, 85, 100),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(30, 90, 180),
                selected_bg: Color::Rgb(60, 75, 110),
                tab_active_bg: Color::Rgb(40, 100, 210),
                tab_inactive_bg: Color::Rgb(35, 40, 50),
                button_bg: Color::Rgb(50, 58, 75),
                button_fg: Color::Rgb(240, 240, 250),
            },
            ThemeMode::Nord => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(46, 52, 64),
                border: Color::Rgb(76, 86, 106),
                border_focused: Color::Rgb(136, 192, 208),
                primary: Color::Rgb(136, 192, 208),     // Frost cyan #88C0D0
                secondary: Color::Rgb(163, 190, 140),   // Aurora green #A3BE8C
                accent: Color::Rgb(235, 203, 139),      // Aurora yellow #EBCB8B
                danger: Color::Rgb(191, 97, 106),       // Aurora red #BF616A
                text_normal: Color::Rgb(236, 239, 244), // Snow storm #ECEFF4
                text_muted: Color::Rgb(216, 222, 233),  // #D8DEE9
                text_dim: Color::Rgb(140, 150, 170),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(94, 129, 172),     // Frost dark blue #5E81AC
                selected_bg: Color::Rgb(67, 76, 94),    // Polar night light #434C5E
                tab_active_bg: Color::Rgb(94, 129, 172),
                tab_inactive_bg: Color::Rgb(46, 52, 64),
                button_bg: Color::Rgb(67, 76, 94),
                button_fg: Color::Rgb(236, 239, 244),
            },
            ThemeMode::Dracula => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(40, 42, 54),
                border: Color::Rgb(68, 71, 90),
                border_focused: Color::Rgb(189, 147, 249),
                primary: Color::Rgb(189, 147, 249),     // Dracula Purple #BD93F9
                secondary: Color::Rgb(80, 250, 123),    // Dracula Green #50FA7B
                accent: Color::Rgb(255, 121, 198),      // Dracula Pink #FF79C6
                danger: Color::Rgb(255, 85, 85),        // Dracula Red #FF5555
                text_normal: Color::Rgb(248, 248, 242), // Dracula Foreground #F8F8F2
                text_muted: Color::Rgb(180, 185, 205),
                text_dim: Color::Rgb(98, 114, 164),     // Dracula Comment #6272A4
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(110, 60, 170),
                selected_bg: Color::Rgb(68, 71, 90),
                tab_active_bg: Color::Rgb(139, 90, 210),
                tab_inactive_bg: Color::Rgb(40, 42, 54),
                button_bg: Color::Rgb(68, 71, 90),
                button_fg: Color::Rgb(248, 248, 242),
            },
            ThemeMode::Forest => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(20, 28, 24),
                border: Color::Rgb(48, 68, 55),
                border_focused: Color::Rgb(100, 215, 150),
                primary: Color::Rgb(100, 215, 150),     // Fresh Sage Green
                secondary: Color::Rgb(75, 190, 180),    // Forest Cyan
                accent: Color::Rgb(240, 200, 85),       // Warm Honey Gold
                danger: Color::Rgb(240, 95, 85),        // Warm Coral Red
                text_normal: Color::Rgb(225, 238, 230),
                text_muted: Color::Rgb(140, 165, 150),
                text_dim: Color::Rgb(85, 110, 95),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(35, 110, 75),
                selected_bg: Color::Rgb(40, 70, 55),
                tab_active_bg: Color::Rgb(45, 125, 85),
                tab_inactive_bg: Color::Rgb(25, 40, 32),
                button_bg: Color::Rgb(42, 65, 52),
                button_fg: Color::Rgb(225, 238, 230),
            },
            ThemeMode::Sunset => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(26, 18, 32),
                border: Color::Rgb(70, 45, 85),
                border_focused: Color::Rgb(255, 64, 129),
                primary: Color::Rgb(255, 64, 129),      // Neon Magenta Pink
                secondary: Color::Rgb(0, 229, 255),     // Electric Cyan
                accent: Color::Rgb(255, 171, 0),        // Sunset Amber
                danger: Color::Rgb(255, 23, 68),        // Crimson
                text_normal: Color::Rgb(240, 230, 250),
                text_muted: Color::Rgb(170, 145, 185),
                text_dim: Color::Rgb(105, 80, 120),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(165, 25, 115),
                selected_bg: Color::Rgb(85, 40, 95),
                tab_active_bg: Color::Rgb(185, 35, 125),
                tab_inactive_bg: Color::Rgb(42, 25, 52),
                button_bg: Color::Rgb(70, 42, 85),
                button_fg: Color::Rgb(245, 235, 255),
            },
            ThemeMode::Monokai => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(34, 31, 34),
                border: Color::Rgb(74, 70, 75),
                border_focused: Color::Rgb(255, 216, 102),
                primary: Color::Rgb(255, 216, 102),     // Monokai Yellow #FFD866
                secondary: Color::Rgb(169, 220, 107),   // Monokai Green #A9DC6B
                accent: Color::Rgb(252, 152, 103),      // Monokai Orange #FC9867
                danger: Color::Rgb(255, 97, 136),       // Monokai Red #FF6188
                text_normal: Color::Rgb(247, 241, 255),
                text_muted: Color::Rgb(155, 150, 155),
                text_dim: Color::Rgb(95, 90, 96),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(125, 95, 30),
                selected_bg: Color::Rgb(65, 58, 66),
                tab_active_bg: Color::Rgb(145, 115, 35),
                tab_inactive_bg: Color::Rgb(42, 38, 42),
                button_bg: Color::Rgb(65, 58, 66),
                button_fg: Color::Rgb(247, 241, 255),
            },
            ThemeMode::Solarized => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(0, 43, 54),
                border: Color::Rgb(7, 54, 66),
                border_focused: Color::Rgb(38, 139, 210),
                primary: Color::Rgb(38, 139, 210),      // Solarized Blue #268bd2
                secondary: Color::Rgb(42, 161, 152),    // Solarized Cyan #2aa198
                accent: Color::Rgb(181, 137, 0),        // Solarized Yellow #b58900
                danger: Color::Rgb(220, 50, 47),        // Solarized Red #dc322f
                text_normal: Color::Rgb(131, 148, 150), // #839496
                text_muted: Color::Rgb(101, 123, 131),  // #657b83
                text_dim: Color::Rgb(88, 110, 117),     // #586e75
                text_highlight: Color::Rgb(238, 232, 213),
                today_bg: Color::Rgb(15, 75, 125),
                selected_bg: Color::Rgb(10, 60, 75),
                tab_active_bg: Color::Rgb(20, 85, 135),
                tab_inactive_bg: Color::Rgb(5, 50, 60),
                button_bg: Color::Rgb(7, 54, 66),
                button_fg: Color::Rgb(238, 232, 213),
            },
        }
    }

    // Dynamic instance methods
    pub fn today_style(&self) -> Style {
        Style::default()
            .fg(self.text_highlight)
            .bg(self.today_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_day_style(&self) -> Style {
        Style::default()
            .fg(self.text_highlight)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_tab_active(&self) -> Style {
        Style::default()
            .fg(self.text_highlight)
            .bg(self.tab_active_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_tab_inactive(&self) -> Style {
        Style::default()
            .fg(self.text_muted)
            .bg(self.tab_inactive_bg)
    }

    pub fn button_style(&self) -> Style {
        Style::default()
            .fg(self.button_fg)
            .bg(self.button_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn active_input_style(&self) -> Style {
        Style::default()
            .fg(self.text_highlight)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }
}
