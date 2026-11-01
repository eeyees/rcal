use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    // 1. Dark Modern & Classic
    ModernDark,
    Nord,
    Dracula,
    TokyoNight,
    CatppuccinMocha,
    CatppuccinMacchiato,
    OneDark,
    GitHubDark,
    NightOwl,
    Cobalt2,

    // 2. Nature, Earthy & Atmospheric
    Forest,
    EverforestDark,
    Kanagawa,
    RosePine,
    RosePineMoon,
    Palenight,
    GruvboxDark,
    AyuDark,
    CatppuccinFrappe,

    // 3. Vibrant, Cyberpunk & Retro
    Sunset,
    Monokai,
    Cyberpunk,
    Synthwave84,
    Matrix,
    AyuMirage,
    TokyoNightStorm,
    SolarizedDark,

    // 4. Clean Light Themes
    SolarizedLight,
    CatppuccinLatte,
    TokyoNightLight,
    GruvboxLight,
    OneLight,
    GitHubLight,
    RosePineDawn,
    EverforestLight,
    LightOwl,
    AyuLight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeCategory {
    Dark,
    Nature,
    Vibrant,
    Light,
}

impl ThemeCategory {
    pub const ALL: &'static [ThemeCategory] = &[
        ThemeCategory::Dark,
        ThemeCategory::Nature,
        ThemeCategory::Vibrant,
        ThemeCategory::Light,
    ];

    pub fn label(&self, lang: crate::i18n::Language) -> &'static str {
        match self {
            ThemeCategory::Dark => match lang {
                crate::i18n::Language::Ko => "다크 (Dark)",
                crate::i18n::Language::Ja => "ダーク (Dark)",
                _ => "Dark",
            },
            ThemeCategory::Nature => match lang {
                crate::i18n::Language::Ko => "내추럴 (Nature)",
                crate::i18n::Language::Ja => "ナチュラル (Nature)",
                _ => "Nature",
            },
            ThemeCategory::Vibrant => match lang {
                crate::i18n::Language::Ko => "비비드 (Vibrant)",
                crate::i18n::Language::Ja => "ビビッド (Vibrant)",
                _ => "Vibrant",
            },
            ThemeCategory::Light => match lang {
                crate::i18n::Language::Ko => "라이트 (Light)",
                crate::i18n::Language::Ja => "ライト (Light)",
                _ => "Light",
            },
        }
    }

    pub fn themes(&self) -> &'static [ThemeMode] {
        match self {
            ThemeCategory::Dark => &[
                ThemeMode::ModernDark,
                ThemeMode::Nord,
                ThemeMode::Dracula,
                ThemeMode::TokyoNight,
                ThemeMode::CatppuccinMocha,
                ThemeMode::CatppuccinMacchiato,
                ThemeMode::OneDark,
                ThemeMode::GitHubDark,
                ThemeMode::NightOwl,
                ThemeMode::Cobalt2,
            ],
            ThemeCategory::Nature => &[
                ThemeMode::Forest,
                ThemeMode::EverforestDark,
                ThemeMode::Kanagawa,
                ThemeMode::RosePine,
                ThemeMode::RosePineMoon,
                ThemeMode::Palenight,
                ThemeMode::GruvboxDark,
                ThemeMode::AyuDark,
                ThemeMode::CatppuccinFrappe,
            ],
            ThemeCategory::Vibrant => &[
                ThemeMode::Sunset,
                ThemeMode::Monokai,
                ThemeMode::Cyberpunk,
                ThemeMode::Synthwave84,
                ThemeMode::Matrix,
                ThemeMode::AyuMirage,
                ThemeMode::TokyoNightStorm,
                ThemeMode::SolarizedDark,
            ],
            ThemeCategory::Light => &[
                ThemeMode::SolarizedLight,
                ThemeMode::CatppuccinLatte,
                ThemeMode::TokyoNightLight,
                ThemeMode::GruvboxLight,
                ThemeMode::OneLight,
                ThemeMode::GitHubLight,
                ThemeMode::RosePineDawn,
                ThemeMode::EverforestLight,
                ThemeMode::LightOwl,
                ThemeMode::AyuLight,
            ],
        }
    }
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::ModernDark => "modern_dark",
            ThemeMode::Nord => "nord",
            ThemeMode::Dracula => "dracula",
            ThemeMode::TokyoNight => "tokyo_night",
            ThemeMode::CatppuccinMocha => "catppuccin_mocha",
            ThemeMode::CatppuccinMacchiato => "catppuccin_macchiato",
            ThemeMode::OneDark => "one_dark",
            ThemeMode::GitHubDark => "github_dark",
            ThemeMode::NightOwl => "night_owl",
            ThemeMode::Cobalt2 => "cobalt2",

            ThemeMode::Forest => "forest",
            ThemeMode::EverforestDark => "everforest_dark",
            ThemeMode::Kanagawa => "kanagawa",
            ThemeMode::RosePine => "rose_pine",
            ThemeMode::RosePineMoon => "rose_pine_moon",
            ThemeMode::Palenight => "palenight",
            ThemeMode::GruvboxDark => "gruvbox_dark",
            ThemeMode::AyuDark => "ayu_dark",
            ThemeMode::CatppuccinFrappe => "catppuccin_frappe",

            ThemeMode::Sunset => "sunset",
            ThemeMode::Monokai => "monokai",
            ThemeMode::Cyberpunk => "cyberpunk",
            ThemeMode::Synthwave84 => "synthwave84",
            ThemeMode::Matrix => "matrix",
            ThemeMode::AyuMirage => "ayu_mirage",
            ThemeMode::TokyoNightStorm => "tokyo_night_storm",
            ThemeMode::SolarizedDark => "solarized_dark",

            ThemeMode::SolarizedLight => "solarized_light",
            ThemeMode::CatppuccinLatte => "catppuccin_latte",
            ThemeMode::TokyoNightLight => "tokyo_night_light",
            ThemeMode::GruvboxLight => "gruvbox_light",
            ThemeMode::OneLight => "one_light",
            ThemeMode::GitHubLight => "github_light",
            ThemeMode::RosePineDawn => "rose_pine_dawn",
            ThemeMode::EverforestLight => "everforest_light",
            ThemeMode::LightOwl => "light_owl",
            ThemeMode::AyuLight => "ayu_light",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().replace('-', "_").as_str() {
            "nord" => ThemeMode::Nord,
            "dracula" => ThemeMode::Dracula,
            "tokyo_night" | "tokyonight" => ThemeMode::TokyoNight,
            "tokyo_night_storm" | "tokyonight_storm" => ThemeMode::TokyoNightStorm,
            "tokyo_night_light" | "tokyonight_light" => ThemeMode::TokyoNightLight,
            "catppuccin_mocha" | "catppuccin" | "mocha" => ThemeMode::CatppuccinMocha,
            "catppuccin_macchiato" | "macchiato" => ThemeMode::CatppuccinMacchiato,
            "catppuccin_frappe" | "frappe" => ThemeMode::CatppuccinFrappe,
            "catppuccin_latte" | "latte" => ThemeMode::CatppuccinLatte,
            "one_dark" | "onedark" => ThemeMode::OneDark,
            "one_light" | "onelight" => ThemeMode::OneLight,
            "github_dark" | "github" => ThemeMode::GitHubDark,
            "github_light" => ThemeMode::GitHubLight,
            "night_owl" | "nightowl" => ThemeMode::NightOwl,
            "light_owl" | "lightowl" => ThemeMode::LightOwl,
            "cobalt2" | "cobalt" => ThemeMode::Cobalt2,

            "forest" | "matcha" => ThemeMode::Forest,
            "everforest_dark" | "everforest" => ThemeMode::EverforestDark,
            "everforest_light" => ThemeMode::EverforestLight,
            "kanagawa" | "wave" => ThemeMode::Kanagawa,
            "rose_pine" | "rosepine" => ThemeMode::RosePine,
            "rose_pine_moon" | "rosepine_moon" => ThemeMode::RosePineMoon,
            "rose_pine_dawn" | "rosepine_dawn" => ThemeMode::RosePineDawn,
            "palenight" | "material_palenight" => ThemeMode::Palenight,
            "gruvbox_dark" | "gruvbox" => ThemeMode::GruvboxDark,
            "gruvbox_light" => ThemeMode::GruvboxLight,
            "ayu_dark" | "ayu" => ThemeMode::AyuDark,
            "ayu_mirage" => ThemeMode::AyuMirage,
            "ayu_light" => ThemeMode::AyuLight,

            "sunset" => ThemeMode::Sunset,
            "monokai" => ThemeMode::Monokai,
            "cyberpunk" | "neon" => ThemeMode::Cyberpunk,
            "synthwave84" | "synthwave_84" | "synthwave" => ThemeMode::Synthwave84,
            "matrix" | "hacker" => ThemeMode::Matrix,
            "solarized" | "solarized_dark" => ThemeMode::SolarizedDark,
            "solarized_light" => ThemeMode::SolarizedLight,

            _ => ThemeMode::ModernDark,
        }
    }

    pub fn all() -> &'static [ThemeMode] {
        &[
            // Dark Modern
            ThemeMode::ModernDark,
            ThemeMode::Nord,
            ThemeMode::Dracula,
            ThemeMode::TokyoNight,
            ThemeMode::CatppuccinMocha,
            ThemeMode::CatppuccinMacchiato,
            ThemeMode::OneDark,
            ThemeMode::GitHubDark,
            ThemeMode::NightOwl,
            ThemeMode::Cobalt2,
            // Nature / Earthy
            ThemeMode::Forest,
            ThemeMode::EverforestDark,
            ThemeMode::Kanagawa,
            ThemeMode::RosePine,
            ThemeMode::RosePineMoon,
            ThemeMode::Palenight,
            ThemeMode::GruvboxDark,
            ThemeMode::AyuDark,
            ThemeMode::CatppuccinFrappe,
            // Vibrant / Retro
            ThemeMode::Sunset,
            ThemeMode::Monokai,
            ThemeMode::Cyberpunk,
            ThemeMode::Synthwave84,
            ThemeMode::Matrix,
            ThemeMode::AyuMirage,
            ThemeMode::TokyoNightStorm,
            ThemeMode::SolarizedDark,
            // Light
            ThemeMode::SolarizedLight,
            ThemeMode::CatppuccinLatte,
            ThemeMode::TokyoNightLight,
            ThemeMode::GruvboxLight,
            ThemeMode::OneLight,
            ThemeMode::GitHubLight,
            ThemeMode::RosePineDawn,
            ThemeMode::EverforestLight,
            ThemeMode::LightOwl,
            ThemeMode::AyuLight,
        ]
    }

    pub fn category(&self) -> ThemeCategory {
        match self {
            ThemeMode::ModernDark
            | ThemeMode::Nord
            | ThemeMode::Dracula
            | ThemeMode::TokyoNight
            | ThemeMode::CatppuccinMocha
            | ThemeMode::CatppuccinMacchiato
            | ThemeMode::OneDark
            | ThemeMode::GitHubDark
            | ThemeMode::NightOwl
            | ThemeMode::Cobalt2 => ThemeCategory::Dark,

            ThemeMode::Forest
            | ThemeMode::EverforestDark
            | ThemeMode::Kanagawa
            | ThemeMode::RosePine
            | ThemeMode::RosePineMoon
            | ThemeMode::Palenight
            | ThemeMode::GruvboxDark
            | ThemeMode::AyuDark
            | ThemeMode::CatppuccinFrappe => ThemeCategory::Nature,

            ThemeMode::Sunset
            | ThemeMode::Monokai
            | ThemeMode::Cyberpunk
            | ThemeMode::Synthwave84
            | ThemeMode::Matrix
            | ThemeMode::AyuMirage
            | ThemeMode::TokyoNightStorm
            | ThemeMode::SolarizedDark => ThemeCategory::Vibrant,

            ThemeMode::SolarizedLight
            | ThemeMode::CatppuccinLatte
            | ThemeMode::TokyoNightLight
            | ThemeMode::GruvboxLight
            | ThemeMode::OneLight
            | ThemeMode::GitHubLight
            | ThemeMode::RosePineDawn
            | ThemeMode::EverforestLight
            | ThemeMode::LightOwl
            | ThemeMode::AyuLight => ThemeCategory::Light,
        }
    }

    pub fn is_light(&self) -> bool {
        self.category() == ThemeCategory::Light
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
            ThemeMode::TokyoNight => match lang {
                crate::i18n::Language::Ko => "도쿄 나이트 (Tokyo Night)",
                crate::i18n::Language::Ja => "東京ナイト (Tokyo Night)",
                _ => "Tokyo Night",
            },
            ThemeMode::CatppuccinMocha => match lang {
                crate::i18n::Language::Ko => "캣푸친 모카 (Catppuccin Mocha)",
                crate::i18n::Language::Ja => "カプチーノモカ (Catppuccin Mocha)",
                _ => "Catppuccin Mocha",
            },
            ThemeMode::CatppuccinMacchiato => match lang {
                crate::i18n::Language::Ko => "캣푸친 마키아토 (Catppuccin)",
                crate::i18n::Language::Ja => "カプチーノマキアート (Catppuccin)",
                _ => "Catppuccin Macchiato",
            },
            ThemeMode::OneDark => match lang {
                crate::i18n::Language::Ko => "원 다크 (One Dark)",
                crate::i18n::Language::Ja => "ワンダーク (One Dark)",
                _ => "One Dark",
            },
            ThemeMode::GitHubDark => match lang {
                crate::i18n::Language::Ko => "깃허브 다크 (GitHub Dark)",
                crate::i18n::Language::Ja => "GitHubダーク (GitHub Dark)",
                _ => "GitHub Dark",
            },
            ThemeMode::NightOwl => match lang {
                crate::i18n::Language::Ko => "나이트 아울 (Night Owl)",
                crate::i18n::Language::Ja => "ナイトオウル (Night Owl)",
                _ => "Night Owl",
            },
            ThemeMode::Cobalt2 => match lang {
                crate::i18n::Language::Ko => "코발트2 (Cobalt2)",
                crate::i18n::Language::Ja => "コバルト2 (Cobalt2)",
                _ => "Cobalt2",
            },

            ThemeMode::Forest => match lang {
                crate::i18n::Language::Ko => "포레스트 (Forest)",
                crate::i18n::Language::Ja => "フォレスト (Forest)",
                _ => "Forest",
            },
            ThemeMode::EverforestDark => match lang {
                crate::i18n::Language::Ko => "에버포레스트 다크 (Everforest)",
                crate::i18n::Language::Ja => "エバーフォレスト (Everforest)",
                _ => "Everforest Dark",
            },
            ThemeMode::Kanagawa => match lang {
                crate::i18n::Language::Ko => "카나가와 (Kanagawa)",
                crate::i18n::Language::Ja => "神奈川 (Kanagawa)",
                _ => "Kanagawa",
            },
            ThemeMode::RosePine => match lang {
                crate::i18n::Language::Ko => "로제 파인 (Rose Pine)",
                crate::i18n::Language::Ja => "ロゼパイン (Rose Pine)",
                _ => "Rose Pine",
            },
            ThemeMode::RosePineMoon => match lang {
                crate::i18n::Language::Ko => "로제 파인 문 (Rose Pine Moon)",
                crate::i18n::Language::Ja => "ロゼパインムーン (Rose Pine Moon)",
                _ => "Rose Pine Moon",
            },
            ThemeMode::Palenight => match lang {
                crate::i18n::Language::Ko => "머티리얼 페일나이트 (Palenight)",
                crate::i18n::Language::Ja => "ペールナイト (Palenight)",
                _ => "Palenight",
            },
            ThemeMode::GruvboxDark => match lang {
                crate::i18n::Language::Ko => "그루브박스 다크 (Gruvbox)",
                crate::i18n::Language::Ja => "グルーヴボックス (Gruvbox)",
                _ => "Gruvbox Dark",
            },
            ThemeMode::AyuDark => match lang {
                crate::i18n::Language::Ko => "아유 다크 (Ayu Dark)",
                crate::i18n::Language::Ja => "Ayuダーク (Ayu Dark)",
                _ => "Ayu Dark",
            },
            ThemeMode::CatppuccinFrappe => match lang {
                crate::i18n::Language::Ko => "캣푸친 프라페 (Catppuccin)",
                crate::i18n::Language::Ja => "カプチーノフラッペ (Catppuccin)",
                _ => "Catppuccin Frappe",
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
            ThemeMode::Cyberpunk => match lang {
                crate::i18n::Language::Ko => "사이버펑크 네온 (Cyberpunk)",
                crate::i18n::Language::Ja => "サイバーパンク (Cyberpunk)",
                _ => "Cyberpunk",
            },
            ThemeMode::Synthwave84 => match lang {
                crate::i18n::Language::Ko => "신스웨이브 '84 (Synthwave)",
                crate::i18n::Language::Ja => "シンセウェーブ '84 (Synthwave)",
                _ => "Synthwave '84",
            },
            ThemeMode::Matrix => match lang {
                crate::i18n::Language::Ko => "매트릭스 CRT (Matrix)",
                crate::i18n::Language::Ja => "マトリックス (Matrix)",
                _ => "Matrix",
            },
            ThemeMode::AyuMirage => match lang {
                crate::i18n::Language::Ko => "아유 미라지 (Ayu Mirage)",
                crate::i18n::Language::Ja => "Ayuミラージュ (Ayu Mirage)",
                _ => "Ayu Mirage",
            },
            ThemeMode::TokyoNightStorm => match lang {
                crate::i18n::Language::Ko => "도쿄 나이트 스톰 (Storm)",
                crate::i18n::Language::Ja => "東京ナイトストーム (Storm)",
                _ => "Tokyo Night Storm",
            },
            ThemeMode::SolarizedDark => match lang {
                crate::i18n::Language::Ko => "솔라라이즈드 다크 (Solarized)",
                crate::i18n::Language::Ja => "ソーラライズドダーク",
                _ => "Solarized Dark",
            },

            ThemeMode::SolarizedLight => match lang {
                crate::i18n::Language::Ko => "솔라라이즈드 라이트 (Light)",
                crate::i18n::Language::Ja => "ソーラライズドライト",
                _ => "Solarized Light",
            },
            ThemeMode::CatppuccinLatte => match lang {
                crate::i18n::Language::Ko => "캣푸친 라떼 (Catppuccin Light)",
                crate::i18n::Language::Ja => "カプチーノラテ (Light)",
                _ => "Catppuccin Latte",
            },
            ThemeMode::TokyoNightLight => match lang {
                crate::i18n::Language::Ko => "도쿄 나이트 라이트 (Light)",
                crate::i18n::Language::Ja => "東京ナイトライト",
                _ => "Tokyo Night Light",
            },
            ThemeMode::GruvboxLight => match lang {
                crate::i18n::Language::Ko => "그루브박스 라이트 (Gruvbox)",
                crate::i18n::Language::Ja => "グルーヴボックスライト",
                _ => "Gruvbox Light",
            },
            ThemeMode::OneLight => match lang {
                crate::i18n::Language::Ko => "원 라이트 (One Light)",
                crate::i18n::Language::Ja => "ワンライト (One Light)",
                _ => "One Light",
            },
            ThemeMode::GitHubLight => match lang {
                crate::i18n::Language::Ko => "깃허브 라이트 (GitHub Light)",
                crate::i18n::Language::Ja => "GitHubライト (GitHub Light)",
                _ => "GitHub Light",
            },
            ThemeMode::RosePineDawn => match lang {
                crate::i18n::Language::Ko => "로제 파인 던 (Rose Dawn)",
                crate::i18n::Language::Ja => "ロゼパインドーン",
                _ => "Rose Pine Dawn",
            },
            ThemeMode::EverforestLight => match lang {
                crate::i18n::Language::Ko => "에버포레스트 라이트 (Light)",
                crate::i18n::Language::Ja => "エバーフォレストライト",
                _ => "Everforest Light",
            },
            ThemeMode::LightOwl => match lang {
                crate::i18n::Language::Ko => "라이트 아울 (Light Owl)",
                crate::i18n::Language::Ja => "ライトオウル (Light Owl)",
                _ => "Light Owl",
            },
            ThemeMode::AyuLight => match lang {
                crate::i18n::Language::Ko => "아유 라이트 (Ayu Light)",
                crate::i18n::Language::Ja => "Ayuライト (Ayu Light)",
                _ => "Ayu Light",
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
            ThemeMode::TokyoNight => match lang {
                crate::i18n::Language::Ko => "도쿄나이트",
                crate::i18n::Language::Ja => "東京ナイト",
                _ => "TokyoNight",
            },
            ThemeMode::CatppuccinMocha => match lang {
                crate::i18n::Language::Ko => "캣-모카",
                crate::i18n::Language::Ja => "モカ",
                _ => "Cat-Mocha",
            },
            ThemeMode::CatppuccinMacchiato => match lang {
                crate::i18n::Language::Ko => "캣-마키아토",
                crate::i18n::Language::Ja => "マキアート",
                _ => "Cat-Macchiato",
            },
            ThemeMode::OneDark => match lang {
                crate::i18n::Language::Ko => "원다크",
                crate::i18n::Language::Ja => "ワンダーク",
                _ => "OneDark",
            },
            ThemeMode::GitHubDark => match lang {
                crate::i18n::Language::Ko => "깃허브다크",
                crate::i18n::Language::Ja => "GHダーク",
                _ => "GitHubDark",
            },
            ThemeMode::NightOwl => match lang {
                crate::i18n::Language::Ko => "나이트아울",
                crate::i18n::Language::Ja => "ナイトオウル",
                _ => "NightOwl",
            },
            ThemeMode::Cobalt2 => match lang {
                crate::i18n::Language::Ko => "코발트2",
                crate::i18n::Language::Ja => "コバルト2",
                _ => "Cobalt2",
            },

            ThemeMode::Forest => match lang {
                crate::i18n::Language::Ko => "포레스트",
                crate::i18n::Language::Ja => "フォレスト",
                _ => "Forest",
            },
            ThemeMode::EverforestDark => match lang {
                crate::i18n::Language::Ko => "에버포레스트",
                crate::i18n::Language::Ja => "エバーF",
                _ => "Everforest",
            },
            ThemeMode::Kanagawa => match lang {
                crate::i18n::Language::Ko => "카나가와",
                crate::i18n::Language::Ja => "神奈川",
                _ => "Kanagawa",
            },
            ThemeMode::RosePine => match lang {
                crate::i18n::Language::Ko => "로제파인",
                crate::i18n::Language::Ja => "ロゼパイン",
                _ => "RosePine",
            },
            ThemeMode::RosePineMoon => match lang {
                crate::i18n::Language::Ko => "로제문",
                crate::i18n::Language::Ja => "ロゼムーン",
                _ => "RoseMoon",
            },
            ThemeMode::Palenight => match lang {
                crate::i18n::Language::Ko => "페일나이트",
                crate::i18n::Language::Ja => "ペールN",
                _ => "Palenight",
            },
            ThemeMode::GruvboxDark => match lang {
                crate::i18n::Language::Ko => "그루브다크",
                crate::i18n::Language::Ja => "グルーヴD",
                _ => "GruvDark",
            },
            ThemeMode::AyuDark => match lang {
                crate::i18n::Language::Ko => "아유다크",
                crate::i18n::Language::Ja => "Ayuダーク",
                _ => "AyuDark",
            },
            ThemeMode::CatppuccinFrappe => match lang {
                crate::i18n::Language::Ko => "캣-프라페",
                crate::i18n::Language::Ja => "フラッペ",
                _ => "Cat-Frappe",
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
            ThemeMode::Cyberpunk => match lang {
                crate::i18n::Language::Ko => "사이버펑크",
                crate::i18n::Language::Ja => "サイバーP",
                _ => "Cyberpunk",
            },
            ThemeMode::Synthwave84 => match lang {
                crate::i18n::Language::Ko => "신스웨이브",
                crate::i18n::Language::Ja => "シンセW",
                _ => "Synthwave",
            },
            ThemeMode::Matrix => match lang {
                crate::i18n::Language::Ko => "매트릭스",
                crate::i18n::Language::Ja => "マトリックス",
                _ => "Matrix",
            },
            ThemeMode::AyuMirage => match lang {
                crate::i18n::Language::Ko => "아유미라지",
                crate::i18n::Language::Ja => "Ayuミラージュ",
                _ => "AyuMirage",
            },
            ThemeMode::TokyoNightStorm => match lang {
                crate::i18n::Language::Ko => "도쿄스톰",
                crate::i18n::Language::Ja => "東京ストーム",
                _ => "TokyoStorm",
            },
            ThemeMode::SolarizedDark => match lang {
                crate::i18n::Language::Ko => "솔라다크",
                crate::i18n::Language::Ja => "ソーラーD",
                _ => "SolarDark",
            },

            ThemeMode::SolarizedLight => match lang {
                crate::i18n::Language::Ko => "솔라라이트",
                crate::i18n::Language::Ja => "ソーラーL",
                _ => "SolarLight",
            },
            ThemeMode::CatppuccinLatte => match lang {
                crate::i18n::Language::Ko => "캣-라떼",
                crate::i18n::Language::Ja => "ラテ",
                _ => "Cat-Latte",
            },
            ThemeMode::TokyoNightLight => match lang {
                crate::i18n::Language::Ko => "도쿄라이트",
                crate::i18n::Language::Ja => "東京ライト",
                _ => "TokyoLight",
            },
            ThemeMode::GruvboxLight => match lang {
                crate::i18n::Language::Ko => "그루브라이트",
                crate::i18n::Language::Ja => "グルーヴL",
                _ => "GruvLight",
            },
            ThemeMode::OneLight => match lang {
                crate::i18n::Language::Ko => "원라이트",
                crate::i18n::Language::Ja => "ワンライト",
                _ => "OneLight",
            },
            ThemeMode::GitHubLight => match lang {
                crate::i18n::Language::Ko => "깃허브라이트",
                crate::i18n::Language::Ja => "GHライト",
                _ => "GitHubLight",
            },
            ThemeMode::RosePineDawn => match lang {
                crate::i18n::Language::Ko => "로제던",
                crate::i18n::Language::Ja => "ロゼドーン",
                _ => "RoseDawn",
            },
            ThemeMode::EverforestLight => match lang {
                crate::i18n::Language::Ko => "에버라이트",
                crate::i18n::Language::Ja => "エバーL",
                _ => "EverLight",
            },
            ThemeMode::LightOwl => match lang {
                crate::i18n::Language::Ko => "라이트아울",
                crate::i18n::Language::Ja => "ライトオウル",
                _ => "LightOwl",
            },
            ThemeMode::AyuLight => match lang {
                crate::i18n::Language::Ko => "아유라이트",
                crate::i18n::Language::Ja => "Ayuライト",
                _ => "AyuLight",
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

    pub const PRIMARY: Color = Color::Rgb(80, 160, 255);
    pub const SECONDARY: Color = Color::Rgb(70, 205, 140);
    pub const ACCENT: Color = Color::Rgb(250, 190, 80);
    pub const DANGER: Color = Color::Rgb(255, 90, 90);

    pub const TEXT_NORMAL: Color = Color::Rgb(220, 225, 235);
    pub const TEXT_MUTED: Color = Color::Rgb(130, 135, 150);
    pub const TEXT_DIM: Color = Color::Rgb(80, 85, 100);
    pub const TEXT_HIGHLIGHT: Color = Color::Rgb(255, 255, 255);

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            // ==========================================
            // 1. Dark Modern & Classic
            // ==========================================
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
                text_muted: Color::Rgb(140, 145, 165),
                text_dim: Color::Rgb(90, 95, 110),
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
                text_muted: Color::Rgb(189, 195, 215),
                text_dim: Color::Rgb(98, 114, 164),     // Dracula Comment #6272A4
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(110, 60, 170),
                selected_bg: Color::Rgb(68, 71, 90),
                tab_active_bg: Color::Rgb(139, 90, 210),
                tab_inactive_bg: Color::Rgb(40, 42, 54),
                button_bg: Color::Rgb(68, 71, 90),
                button_fg: Color::Rgb(248, 248, 242),
            },
            ThemeMode::TokyoNight => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(26, 27, 38),
                border: Color::Rgb(65, 72, 104),
                border_focused: Color::Rgb(122, 162, 247),
                primary: Color::Rgb(122, 162, 247),     // #7AA2F7
                secondary: Color::Rgb(158, 206, 106),   // #9ECE6A
                accent: Color::Rgb(224, 175, 104),      // #E0AF68
                danger: Color::Rgb(247, 118, 142),      // #F7768E
                text_normal: Color::Rgb(192, 202, 245), // #C0CAF5
                text_muted: Color::Rgb(154, 165, 206),  // #9AA5CE
                text_dim: Color::Rgb(86, 95, 137),      // #565F89
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(45, 60, 110),
                selected_bg: Color::Rgb(50, 56, 85),
                tab_active_bg: Color::Rgb(61, 89, 161),
                tab_inactive_bg: Color::Rgb(32, 34, 48),
                button_bg: Color::Rgb(44, 50, 75),
                button_fg: Color::Rgb(192, 202, 245),
            },
            ThemeMode::CatppuccinMocha => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(30, 30, 46),       // #1E1E2E
                border: Color::Rgb(69, 71, 90),         // #45475A
                border_focused: Color::Rgb(137, 180, 250), // #89B4FA
                primary: Color::Rgb(137, 180, 250),
                secondary: Color::Rgb(166, 227, 161),   // #A6E3A1
                accent: Color::Rgb(249, 226, 175),      // #F9E2AF
                danger: Color::Rgb(243, 139, 168),      // #F38BA8
                text_normal: Color::Rgb(205, 214, 244), // #CDD6F4
                text_muted: Color::Rgb(166, 173, 200),  // #A6ADC8
                text_dim: Color::Rgb(108, 112, 134),    // #6C7086
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(49, 50, 68),
                selected_bg: Color::Rgb(69, 71, 90),
                tab_active_bg: Color::Rgb(137, 180, 250),
                tab_inactive_bg: Color::Rgb(24, 24, 37),
                button_bg: Color::Rgb(49, 50, 68),
                button_fg: Color::Rgb(205, 214, 244),
            },
            ThemeMode::CatppuccinMacchiato => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(36, 39, 58),       // #24273A
                border: Color::Rgb(73, 77, 100),        // #494D64
                border_focused: Color::Rgb(138, 173, 244), // #8AADF4
                primary: Color::Rgb(138, 173, 244),
                secondary: Color::Rgb(166, 218, 149),   // #A6DA95
                accent: Color::Rgb(238, 212, 159),      // #EED49F
                danger: Color::Rgb(237, 135, 150),      // #ED8796
                text_normal: Color::Rgb(202, 211, 245), // #CAD3F5
                text_muted: Color::Rgb(165, 173, 203),
                text_dim: Color::Rgb(110, 115, 141),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(54, 58, 79),
                selected_bg: Color::Rgb(73, 77, 100),
                tab_active_bg: Color::Rgb(138, 173, 244),
                tab_inactive_bg: Color::Rgb(30, 32, 48),
                button_bg: Color::Rgb(54, 58, 79),
                button_fg: Color::Rgb(202, 211, 245),
            },
            ThemeMode::OneDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(40, 44, 52),       // #282C34
                border: Color::Rgb(62, 68, 81),
                border_focused: Color::Rgb(97, 175, 239), // #61AFEF
                primary: Color::Rgb(97, 175, 239),
                secondary: Color::Rgb(152, 195, 121),   // #98C379
                accent: Color::Rgb(229, 192, 123),      // #E5C07B
                danger: Color::Rgb(224, 108, 117),      // #E06C75
                text_normal: Color::Rgb(171, 178, 191), // #ABB2BF
                text_muted: Color::Rgb(140, 148, 163),
                text_dim: Color::Rgb(92, 99, 112),      // #5C6370
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(44, 75, 120),
                selected_bg: Color::Rgb(59, 66, 82),
                tab_active_bg: Color::Rgb(64, 100, 160),
                tab_inactive_bg: Color::Rgb(33, 37, 43),
                button_bg: Color::Rgb(50, 56, 66),
                button_fg: Color::Rgb(171, 178, 191),
            },
            ThemeMode::GitHubDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(22, 27, 34),       // #161B22
                border: Color::Rgb(48, 54, 61),         // #30363D
                border_focused: Color::Rgb(88, 166, 255), // #58A6FF
                primary: Color::Rgb(88, 166, 255),
                secondary: Color::Rgb(63, 185, 80),     // #3FB950
                accent: Color::Rgb(210, 153, 34),       // #D29922
                danger: Color::Rgb(248, 81, 73),        // #F85149
                text_normal: Color::Rgb(201, 209, 217), // #C9D1D9
                text_muted: Color::Rgb(139, 148, 158),  // #8B949E
                text_dim: Color::Rgb(72, 79, 88),       // #484F58
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(31, 58, 95),
                selected_bg: Color::Rgb(38, 44, 54),
                tab_active_bg: Color::Rgb(31, 111, 235),
                tab_inactive_bg: Color::Rgb(13, 17, 23),
                button_bg: Color::Rgb(33, 38, 45),
                button_fg: Color::Rgb(201, 209, 217),
            },
            ThemeMode::NightOwl => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(1, 22, 39),        // #011627
                border: Color::Rgb(28, 55, 80),
                border_focused: Color::Rgb(130, 170, 255), // #82AAFF
                primary: Color::Rgb(130, 170, 255),
                secondary: Color::Rgb(34, 218, 110),    // #22DA6E
                accent: Color::Rgb(255, 203, 139),      // #FFCB8B
                danger: Color::Rgb(239, 83, 80),        // #EF5350
                text_normal: Color::Rgb(214, 222, 235), // #D6DEEB
                text_muted: Color::Rgb(150, 170, 195),
                text_dim: Color::Rgb(95, 126, 150),     // #5F7E97
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(20, 60, 110),
                selected_bg: Color::Rgb(15, 45, 75),
                tab_active_bg: Color::Rgb(30, 80, 150),
                tab_inactive_bg: Color::Rgb(1, 16, 28),
                button_bg: Color::Rgb(11, 41, 66),
                button_fg: Color::Rgb(214, 222, 235),
            },
            ThemeMode::Cobalt2 => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(25, 53, 78),       // #19354E
                border: Color::Rgb(20, 85, 135),
                border_focused: Color::Rgb(255, 197, 35), // #FFC523
                primary: Color::Rgb(255, 197, 35),
                secondary: Color::Rgb(59, 219, 139),    // #3BDB8B
                accent: Color::Rgb(0, 168, 255),        // #00A8FF
                danger: Color::Rgb(255, 78, 66),
                text_normal: Color::Rgb(255, 255, 255),
                text_muted: Color::Rgb(170, 195, 220),
                text_dim: Color::Rgb(90, 125, 155),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(35, 80, 130),
                selected_bg: Color::Rgb(30, 65, 100),
                tab_active_bg: Color::Rgb(0, 136, 204),
                tab_inactive_bg: Color::Rgb(18, 40, 60),
                button_bg: Color::Rgb(30, 65, 100),
                button_fg: Color::Rgb(255, 255, 255),
            },

            // ==========================================
            // 2. Nature, Earthy & Atmospheric
            // ==========================================
            ThemeMode::Forest => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(20, 28, 24),
                border: Color::Rgb(48, 68, 55),
                border_focused: Color::Rgb(100, 215, 150),
                primary: Color::Rgb(100, 215, 150),
                secondary: Color::Rgb(75, 190, 180),
                accent: Color::Rgb(240, 200, 85),
                danger: Color::Rgb(240, 95, 85),
                text_normal: Color::Rgb(225, 238, 230),
                text_muted: Color::Rgb(150, 175, 160),
                text_dim: Color::Rgb(95, 120, 105),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(35, 110, 75),
                selected_bg: Color::Rgb(40, 70, 55),
                tab_active_bg: Color::Rgb(45, 125, 85),
                tab_inactive_bg: Color::Rgb(25, 40, 32),
                button_bg: Color::Rgb(42, 65, 52),
                button_fg: Color::Rgb(225, 238, 230),
            },
            ThemeMode::EverforestDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(45, 53, 59),       // #2D353B
                border: Color::Rgb(71, 82, 88),         // #475258
                border_focused: Color::Rgb(167, 192, 128), // #A7C080
                primary: Color::Rgb(167, 192, 128),
                secondary: Color::Rgb(131, 192, 146),   // #83C092
                accent: Color::Rgb(219, 188, 127),      // #DBBC7F
                danger: Color::Rgb(230, 126, 128),      // #E67E80
                text_normal: Color::Rgb(211, 198, 170), // #D3C6AA
                text_muted: Color::Rgb(157, 169, 160),
                text_dim: Color::Rgb(112, 122, 115),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(60, 85, 65),
                selected_bg: Color::Rgb(58, 68, 71),
                tab_active_bg: Color::Rgb(78, 105, 80),
                tab_inactive_bg: Color::Rgb(35, 42, 46),
                button_bg: Color::Rgb(58, 68, 71),
                button_fg: Color::Rgb(211, 198, 170),
            },
            ThemeMode::Kanagawa => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(31, 31, 40),       // #1F1F28
                border: Color::Rgb(54, 54, 70),         // #363646
                border_focused: Color::Rgb(126, 156, 216), // #7E9CD8
                primary: Color::Rgb(126, 156, 216),
                secondary: Color::Rgb(152, 187, 108),   // #98BB6C
                accent: Color::Rgb(230, 195, 132),      // #E6C384
                danger: Color::Rgb(232, 131, 132),      // #E88384
                text_normal: Color::Rgb(220, 223, 228), // #DCD7BA
                text_muted: Color::Rgb(155, 160, 175),
                text_dim: Color::Rgb(114, 113, 105),    // #727169
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(45, 65, 95),
                selected_bg: Color::Rgb(42, 42, 58),
                tab_active_bg: Color::Rgb(54, 75, 110),
                tab_inactive_bg: Color::Rgb(22, 22, 29),
                button_bg: Color::Rgb(42, 42, 58),
                button_fg: Color::Rgb(220, 223, 228),
            },
            ThemeMode::RosePine => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(25, 23, 36),       // #191724
                border: Color::Rgb(38, 35, 58),         // #26233A
                border_focused: Color::Rgb(156, 207, 216), // #9CCFD8
                primary: Color::Rgb(235, 188, 186),     // #EBBCBA
                secondary: Color::Rgb(49, 116, 143),    // #31748F
                accent: Color::Rgb(246, 193, 119),      // #F6C177
                danger: Color::Rgb(235, 111, 146),      // #EB6F92
                text_normal: Color::Rgb(224, 222, 244), // #E0DEF4
                text_muted: Color::Rgb(144, 140, 170),  // #908CAA
                text_dim: Color::Rgb(110, 106, 134),    // #6E6A86
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(60, 45, 75),
                selected_bg: Color::Rgb(42, 39, 63),
                tab_active_bg: Color::Rgb(80, 55, 95),
                tab_inactive_bg: Color::Rgb(20, 18, 30),
                button_bg: Color::Rgb(42, 39, 63),
                button_fg: Color::Rgb(224, 222, 244),
            },
            ThemeMode::RosePineMoon => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(35, 33, 54),       // #232136
                border: Color::Rgb(57, 53, 82),         // #393552
                border_focused: Color::Rgb(156, 207, 216),
                primary: Color::Rgb(234, 154, 151),     // #EA9A97
                secondary: Color::Rgb(62, 143, 176),    // #3E8FB0
                accent: Color::Rgb(246, 193, 119),
                danger: Color::Rgb(235, 111, 146),
                text_normal: Color::Rgb(224, 222, 244),
                text_muted: Color::Rgb(144, 140, 170),
                text_dim: Color::Rgb(110, 106, 134),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(70, 50, 85),
                selected_bg: Color::Rgb(57, 53, 82),
                tab_active_bg: Color::Rgb(85, 60, 105),
                tab_inactive_bg: Color::Rgb(27, 25, 42),
                button_bg: Color::Rgb(57, 53, 82),
                button_fg: Color::Rgb(224, 222, 244),
            },
            ThemeMode::Palenight => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(41, 45, 62),       // #292D3E
                border: Color::Rgb(68, 75, 106),
                border_focused: Color::Rgb(130, 170, 255), // #82AAFF
                primary: Color::Rgb(130, 170, 255),
                secondary: Color::Rgb(195, 232, 141),   // #C3E88D
                accent: Color::Rgb(255, 203, 107),      // #FFCB6B
                danger: Color::Rgb(255, 83, 112),       // #FF5370
                text_normal: Color::Rgb(238, 245, 255),
                text_muted: Color::Rgb(170, 175, 205),
                text_dim: Color::Rgb(103, 110, 149),    // #676E95
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(55, 65, 110),
                selected_bg: Color::Rgb(55, 60, 85),
                tab_active_bg: Color::Rgb(75, 90, 145),
                tab_inactive_bg: Color::Rgb(30, 33, 46),
                button_bg: Color::Rgb(50, 55, 78),
                button_fg: Color::Rgb(238, 245, 255),
            },
            ThemeMode::GruvboxDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(40, 40, 40),       // #282828
                border: Color::Rgb(80, 73, 69),         // #504945
                border_focused: Color::Rgb(250, 189, 47), // #FABD2F
                primary: Color::Rgb(250, 189, 47),
                secondary: Color::Rgb(184, 187, 38),    // #B8BB26
                accent: Color::Rgb(254, 128, 25),       // #FE8019
                danger: Color::Rgb(251, 73, 52),        // #FB4934
                text_normal: Color::Rgb(235, 219, 178), // #EBDBB2
                text_muted: Color::Rgb(168, 153, 132),  // #A89984
                text_dim: Color::Rgb(124, 111, 100),    // #7C6F64
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(102, 92, 84),
                selected_bg: Color::Rgb(60, 56, 54),
                tab_active_bg: Color::Rgb(102, 92, 84),
                tab_inactive_bg: Color::Rgb(29, 32, 33),
                button_bg: Color::Rgb(60, 56, 54),
                button_fg: Color::Rgb(235, 219, 178),
            },
            ThemeMode::AyuDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(15, 20, 25),       // #0F1419
                border: Color::Rgb(40, 48, 58),
                border_focused: Color::Rgb(255, 180, 84), // #FFB454
                primary: Color::Rgb(255, 180, 84),
                secondary: Color::Rgb(149, 230, 150),   // #95E696
                accent: Color::Rgb(57, 186, 230),       // #39BAE6
                danger: Color::Rgb(255, 51, 51),
                text_normal: Color::Rgb(225, 230, 235), // #E6E1CF
                text_muted: Color::Rgb(160, 165, 175),
                text_dim: Color::Rgb(92, 103, 115),     // #5C6773
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(85, 60, 25),
                selected_bg: Color::Rgb(30, 38, 48),
                tab_active_bg: Color::Rgb(105, 75, 30),
                tab_inactive_bg: Color::Rgb(10, 14, 18),
                button_bg: Color::Rgb(30, 38, 48),
                button_fg: Color::Rgb(225, 230, 235),
            },
            ThemeMode::CatppuccinFrappe => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(48, 52, 70),       // #303446
                border: Color::Rgb(81, 87, 109),        // #51576D
                border_focused: Color::Rgb(140, 170, 238), // #8CAAEE
                primary: Color::Rgb(140, 170, 238),
                secondary: Color::Rgb(166, 209, 137),   // #A6D189
                accent: Color::Rgb(229, 200, 144),      // #E5C890
                danger: Color::Rgb(231, 130, 132),      // #E78284
                text_normal: Color::Rgb(198, 208, 245), // #C6D0F5
                text_muted: Color::Rgb(165, 173, 206),
                text_dim: Color::Rgb(115, 121, 148),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(65, 69, 89),
                selected_bg: Color::Rgb(81, 87, 109),
                tab_active_bg: Color::Rgb(140, 170, 238),
                tab_inactive_bg: Color::Rgb(35, 38, 52),
                button_bg: Color::Rgb(65, 69, 89),
                button_fg: Color::Rgb(198, 208, 245),
            },

            // ==========================================
            // 3. Vibrant, Cyberpunk & Retro
            // ==========================================
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
                text_muted: Color::Rgb(180, 155, 195),
                text_dim: Color::Rgb(115, 90, 130),
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
                text_muted: Color::Rgb(170, 165, 170),
                text_dim: Color::Rgb(110, 105, 110),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(125, 95, 30),
                selected_bg: Color::Rgb(65, 58, 66),
                tab_active_bg: Color::Rgb(145, 115, 35),
                tab_inactive_bg: Color::Rgb(42, 38, 42),
                button_bg: Color::Rgb(65, 58, 66),
                button_fg: Color::Rgb(247, 241, 255),
            },
            ThemeMode::Cyberpunk => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(10, 10, 15),
                border: Color::Rgb(50, 45, 75),
                border_focused: Color::Rgb(0, 255, 204),
                primary: Color::Rgb(255, 0, 128),
                secondary: Color::Rgb(0, 255, 204),
                accent: Color::Rgb(255, 255, 0),
                danger: Color::Rgb(255, 0, 55),
                text_normal: Color::Rgb(240, 245, 255),
                text_muted: Color::Rgb(180, 170, 210),
                text_dim: Color::Rgb(105, 95, 135),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(140, 0, 80),
                selected_bg: Color::Rgb(40, 30, 60),
                tab_active_bg: Color::Rgb(180, 0, 100),
                tab_inactive_bg: Color::Rgb(20, 15, 30),
                button_bg: Color::Rgb(50, 35, 75),
                button_fg: Color::Rgb(0, 255, 204),
            },
            ThemeMode::Synthwave84 => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(38, 35, 53),       // #262335
                border: Color::Rgb(70, 55, 90),
                border_focused: Color::Rgb(254, 68, 142), // #FE448E
                primary: Color::Rgb(254, 68, 142),
                secondary: Color::Rgb(54, 249, 246),    // #36F9F6
                accent: Color::Rgb(254, 222, 93),       // #FEDE5D
                danger: Color::Rgb(254, 68, 68),
                text_normal: Color::Rgb(255, 245, 255),
                text_muted: Color::Rgb(180, 160, 205),
                text_dim: Color::Rgb(110, 95, 135),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(120, 30, 110),
                selected_bg: Color::Rgb(65, 45, 80),
                tab_active_bg: Color::Rgb(160, 40, 130),
                tab_inactive_bg: Color::Rgb(28, 24, 40),
                button_bg: Color::Rgb(65, 45, 80),
                button_fg: Color::Rgb(254, 68, 142),
            },
            ThemeMode::Matrix => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(5, 15, 5),
                border: Color::Rgb(20, 60, 20),
                border_focused: Color::Rgb(0, 255, 65),
                primary: Color::Rgb(0, 255, 65),
                secondary: Color::Rgb(50, 205, 50),
                accent: Color::Rgb(0, 190, 45),
                danger: Color::Rgb(255, 60, 60),
                text_normal: Color::Rgb(200, 255, 200),
                text_muted: Color::Rgb(100, 200, 100),
                text_dim: Color::Rgb(30, 110, 30),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(15, 75, 25),
                selected_bg: Color::Rgb(10, 45, 15),
                tab_active_bg: Color::Rgb(0, 130, 35),
                tab_inactive_bg: Color::Rgb(8, 22, 8),
                button_bg: Color::Rgb(12, 45, 12),
                button_fg: Color::Rgb(0, 255, 65),
            },
            ThemeMode::AyuMirage => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(31, 36, 48),       // #1F2430
                border: Color::Rgb(55, 62, 80),
                border_focused: Color::Rgb(255, 204, 102), // #FFCC66
                primary: Color::Rgb(255, 204, 102),
                secondary: Color::Rgb(186, 230, 126),   // #BAE67E
                accent: Color::Rgb(115, 208, 255),      // #73D0FF
                danger: Color::Rgb(255, 102, 102),
                text_normal: Color::Rgb(203, 210, 222), // #CBCCC6
                text_muted: Color::Rgb(155, 165, 180),
                text_dim: Color::Rgb(105, 115, 130),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(70, 60, 40),
                selected_bg: Color::Rgb(45, 52, 68),
                tab_active_bg: Color::Rgb(90, 75, 45),
                tab_inactive_bg: Color::Rgb(22, 26, 35),
                button_bg: Color::Rgb(45, 52, 68),
                button_fg: Color::Rgb(203, 210, 222),
            },
            ThemeMode::TokyoNightStorm => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(36, 40, 59),       // #24283B
                border: Color::Rgb(74, 82, 118),
                border_focused: Color::Rgb(122, 162, 247),
                primary: Color::Rgb(122, 162, 247),
                secondary: Color::Rgb(115, 218, 202),   // #73DACA
                accent: Color::Rgb(255, 158, 100),      // #FF9E64
                danger: Color::Rgb(247, 118, 142),
                text_normal: Color::Rgb(192, 202, 245),
                text_muted: Color::Rgb(154, 165, 206),
                text_dim: Color::Rgb(86, 95, 137),
                text_highlight: Color::Rgb(255, 255, 255),
                today_bg: Color::Rgb(61, 89, 161),      // #3D59A1
                selected_bg: Color::Rgb(52, 60, 88),
                tab_active_bg: Color::Rgb(75, 105, 180),
                tab_inactive_bg: Color::Rgb(28, 31, 46),
                button_bg: Color::Rgb(52, 60, 88),
                button_fg: Color::Rgb(192, 202, 245),
            },
            ThemeMode::SolarizedDark => Self {
                mode,
                bg: Color::Reset,
                panel_bg: Color::Rgb(0, 43, 54),        // #002B36
                border: Color::Rgb(7, 54, 66),          // #073642
                border_focused: Color::Rgb(38, 139, 210), // #268BD2
                primary: Color::Rgb(38, 139, 210),
                secondary: Color::Rgb(42, 161, 152),    // #2AA198
                accent: Color::Rgb(181, 137, 0),        // #B58900
                danger: Color::Rgb(220, 50, 47),        // #DC322F
                text_normal: Color::Rgb(147, 161, 161), // #93A1A1
                text_muted: Color::Rgb(101, 123, 131),  // #657B83
                text_dim: Color::Rgb(88, 110, 117),     // #586E75
                text_highlight: Color::Rgb(238, 232, 213),
                today_bg: Color::Rgb(15, 75, 125),
                selected_bg: Color::Rgb(10, 60, 75),
                tab_active_bg: Color::Rgb(20, 85, 135),
                tab_inactive_bg: Color::Rgb(5, 50, 60),
                button_bg: Color::Rgb(7, 54, 66),
                button_fg: Color::Rgb(238, 232, 213),
            },

            // ==========================================
            // 4. Clean Light Themes
            // ==========================================
            ThemeMode::SolarizedLight => Self {
                mode,
                bg: Color::Rgb(253, 246, 227),          // #FDF6E3
                panel_bg: Color::Rgb(238, 232, 213),    // #EEE8D5
                border: Color::Rgb(203, 194, 169),
                border_focused: Color::Rgb(38, 139, 210),
                primary: Color::Rgb(38, 139, 210),
                secondary: Color::Rgb(42, 161, 152),
                accent: Color::Rgb(181, 137, 0),
                danger: Color::Rgb(220, 50, 47),
                text_normal: Color::Rgb(101, 123, 131), // #657B83
                text_muted: Color::Rgb(131, 148, 150),
                text_dim: Color::Rgb(147, 161, 161),
                text_highlight: Color::Rgb(7, 54, 66),
                today_bg: Color::Rgb(185, 215, 240),
                selected_bg: Color::Rgb(215, 225, 215),
                tab_active_bg: Color::Rgb(38, 139, 210),
                tab_inactive_bg: Color::Rgb(225, 218, 198),
                button_bg: Color::Rgb(225, 218, 198),
                button_fg: Color::Rgb(101, 123, 131),
            },
            ThemeMode::CatppuccinLatte => Self {
                mode,
                bg: Color::Rgb(239, 241, 245),          // #EFF1F5
                panel_bg: Color::Rgb(230, 233, 239),    // #E6E9EF
                border: Color::Rgb(188, 192, 204),      // #BCC0CC
                border_focused: Color::Rgb(30, 102, 245), // #1E66F5
                primary: Color::Rgb(30, 102, 245),
                secondary: Color::Rgb(64, 160, 43),     // #40A02B
                accent: Color::Rgb(223, 142, 29),       // #DF8E1D
                danger: Color::Rgb(210, 15, 57),        // #D20F39
                text_normal: Color::Rgb(76, 79, 105),   // #4C4F69
                text_muted: Color::Rgb(108, 111, 133),  // #6C6F85
                text_dim: Color::Rgb(140, 143, 161),
                text_highlight: Color::Rgb(32, 34, 45),
                today_bg: Color::Rgb(180, 205, 255),
                selected_bg: Color::Rgb(204, 208, 218),
                tab_active_bg: Color::Rgb(30, 102, 245),
                tab_inactive_bg: Color::Rgb(218, 222, 230),
                button_bg: Color::Rgb(218, 222, 230),
                button_fg: Color::Rgb(76, 79, 105),
            },
            ThemeMode::TokyoNightLight => Self {
                mode,
                bg: Color::Rgb(229, 230, 235),
                panel_bg: Color::Rgb(213, 214, 219),
                border: Color::Rgb(168, 171, 180),
                border_focused: Color::Rgb(52, 84, 184),
                primary: Color::Rgb(52, 84, 184),
                secondary: Color::Rgb(51, 105, 30),
                accent: Color::Rgb(143, 94, 21),
                danger: Color::Rgb(140, 43, 66),
                text_normal: Color::Rgb(52, 59, 88),
                text_muted: Color::Rgb(86, 95, 137),
                text_dim: Color::Rgb(120, 128, 150),
                text_highlight: Color::Rgb(20, 24, 38),
                today_bg: Color::Rgb(185, 205, 245),
                selected_bg: Color::Rgb(190, 195, 210),
                tab_active_bg: Color::Rgb(52, 84, 184),
                tab_inactive_bg: Color::Rgb(200, 202, 210),
                button_bg: Color::Rgb(200, 202, 210),
                button_fg: Color::Rgb(52, 59, 88),
            },
            ThemeMode::GruvboxLight => Self {
                mode,
                bg: Color::Rgb(251, 241, 199),          // #FBF1C7
                panel_bg: Color::Rgb(242, 229, 188),    // #F2E5BC
                border: Color::Rgb(213, 196, 161),      // #D5C4A1
                border_focused: Color::Rgb(181, 118, 20), // #B57614
                primary: Color::Rgb(181, 118, 20),
                secondary: Color::Rgb(152, 151, 26),    // #98971A
                accent: Color::Rgb(175, 58, 3),         // #AF3A03
                danger: Color::Rgb(204, 36, 29),        // #CC241D
                text_normal: Color::Rgb(60, 56, 54),    // #3C3836
                text_muted: Color::Rgb(124, 111, 100),  // #7C6F64
                text_dim: Color::Rgb(168, 153, 132),
                text_highlight: Color::Rgb(29, 32, 33),
                today_bg: Color::Rgb(225, 200, 150),
                selected_bg: Color::Rgb(235, 219, 178),
                tab_active_bg: Color::Rgb(181, 118, 20),
                tab_inactive_bg: Color::Rgb(228, 214, 174),
                button_bg: Color::Rgb(228, 214, 174),
                button_fg: Color::Rgb(60, 56, 54),
            },
            ThemeMode::OneLight => Self {
                mode,
                bg: Color::Rgb(250, 250, 250),          // #FAFAFA
                panel_bg: Color::Rgb(234, 234, 234),
                border: Color::Rgb(190, 192, 198),
                border_focused: Color::Rgb(64, 120, 242), // #4078F2
                primary: Color::Rgb(64, 120, 242),
                secondary: Color::Rgb(80, 161, 79),     // #50A14F
                accent: Color::Rgb(193, 132, 1),        // #C18401
                danger: Color::Rgb(228, 86, 73),        // #E45649
                text_normal: Color::Rgb(56, 58, 66),    // #383A42
                text_muted: Color::Rgb(105, 108, 120),
                text_dim: Color::Rgb(160, 161, 167),
                text_highlight: Color::Rgb(20, 20, 24),
                today_bg: Color::Rgb(190, 215, 250),
                selected_bg: Color::Rgb(215, 220, 230),
                tab_active_bg: Color::Rgb(64, 120, 242),
                tab_inactive_bg: Color::Rgb(222, 222, 224),
                button_bg: Color::Rgb(222, 222, 224),
                button_fg: Color::Rgb(56, 58, 66),
            },
            ThemeMode::GitHubLight => Self {
                mode,
                bg: Color::Rgb(255, 255, 255),          // #FFFFFF
                panel_bg: Color::Rgb(246, 248, 250),    // #F6F8FA
                border: Color::Rgb(208, 215, 222),      // #D0D7DE
                border_focused: Color::Rgb(9, 105, 218), // #0969DA
                primary: Color::Rgb(9, 105, 218),
                secondary: Color::Rgb(26, 127, 55),     // #1A7F37
                accent: Color::Rgb(154, 103, 0),        // #9A6700
                danger: Color::Rgb(207, 34, 46),        // #CF222E
                text_normal: Color::Rgb(31, 35, 40),    // #1F2328
                text_muted: Color::Rgb(101, 109, 118),  // #656D76
                text_dim: Color::Rgb(140, 149, 159),
                text_highlight: Color::Rgb(10, 12, 14),
                today_bg: Color::Rgb(190, 220, 255),
                selected_bg: Color::Rgb(234, 238, 242),
                tab_active_bg: Color::Rgb(9, 105, 218),
                tab_inactive_bg: Color::Rgb(234, 238, 242),
                button_bg: Color::Rgb(234, 238, 242),
                button_fg: Color::Rgb(31, 35, 40),
            },
            ThemeMode::RosePineDawn => Self {
                mode,
                bg: Color::Rgb(250, 244, 237),          // #FAF4ED
                panel_bg: Color::Rgb(242, 233, 222),    // #F2E9DE
                border: Color::Rgb(215, 202, 190),
                border_focused: Color::Rgb(40, 105, 131), // #286983
                primary: Color::Rgb(215, 130, 126),     // #D7827E
                secondary: Color::Rgb(40, 105, 131),
                accent: Color::Rgb(234, 157, 52),       // #EA9D34
                danger: Color::Rgb(180, 99, 122),       // #B4637A
                text_normal: Color::Rgb(87, 82, 121),   // #575279
                text_muted: Color::Rgb(121, 117, 147),  // #797593
                text_dim: Color::Rgb(152, 147, 165),
                text_highlight: Color::Rgb(40, 36, 60),
                today_bg: Color::Rgb(235, 210, 210),
                selected_bg: Color::Rgb(225, 215, 205),
                tab_active_bg: Color::Rgb(40, 105, 131),
                tab_inactive_bg: Color::Rgb(230, 220, 210),
                button_bg: Color::Rgb(230, 220, 210),
                button_fg: Color::Rgb(87, 82, 121),
            },
            ThemeMode::EverforestLight => Self {
                mode,
                bg: Color::Rgb(253, 246, 227),          // #FDF6E3
                panel_bg: Color::Rgb(239, 235, 214),    // #EFEBD6
                border: Color::Rgb(205, 197, 175),
                border_focused: Color::Rgb(141, 161, 1), // #8DA101
                primary: Color::Rgb(141, 161, 1),
                secondary: Color::Rgb(58, 148, 107),    // #3A946B
                accent: Color::Rgb(183, 111, 23),       // #B76F17
                danger: Color::Rgb(248, 85, 82),        // #F85552
                text_normal: Color::Rgb(92, 106, 114),  // #5C6A72
                text_muted: Color::Rgb(130, 142, 148),
                text_dim: Color::Rgb(160, 168, 165),
                text_highlight: Color::Rgb(40, 48, 52),
                today_bg: Color::Rgb(210, 230, 195),
                selected_bg: Color::Rgb(225, 220, 200),
                tab_active_bg: Color::Rgb(141, 161, 1),
                tab_inactive_bg: Color::Rgb(226, 220, 198),
                button_bg: Color::Rgb(226, 220, 198),
                button_fg: Color::Rgb(92, 106, 114),
            },
            ThemeMode::LightOwl => Self {
                mode,
                bg: Color::Rgb(240, 240, 240),          // #F0F0F0
                panel_bg: Color::Rgb(255, 255, 255),
                border: Color::Rgb(200, 205, 215),
                border_focused: Color::Rgb(64, 63, 198), // #403FC6
                primary: Color::Rgb(64, 63, 198),
                secondary: Color::Rgb(42, 160, 75),
                accent: Color::Rgb(195, 115, 0),
                danger: Color::Rgb(222, 62, 59),
                text_normal: Color::Rgb(64, 63, 76),    // #403F4C
                text_muted: Color::Rgb(115, 120, 135),
                text_dim: Color::Rgb(150, 155, 170),
                text_highlight: Color::Rgb(20, 20, 28),
                today_bg: Color::Rgb(195, 210, 250),
                selected_bg: Color::Rgb(225, 230, 245),
                tab_active_bg: Color::Rgb(64, 63, 198),
                tab_inactive_bg: Color::Rgb(230, 232, 238),
                button_bg: Color::Rgb(230, 232, 238),
                button_fg: Color::Rgb(64, 63, 76),
            },
            ThemeMode::AyuLight => Self {
                mode,
                bg: Color::Rgb(250, 250, 250),          // #FAFAFA
                panel_bg: Color::Rgb(240, 240, 240),
                border: Color::Rgb(210, 212, 216),
                border_focused: Color::Rgb(242, 153, 58), // #F2993A
                primary: Color::Rgb(242, 153, 58),
                secondary: Color::Rgb(134, 179, 0),     // #86B300
                accent: Color::Rgb(57, 158, 230),       // #399EE6
                danger: Color::Rgb(240, 113, 120),      // #F07178
                text_normal: Color::Rgb(92, 97, 102),   // #5C6166
                text_muted: Color::Rgb(125, 130, 135),
                text_dim: Color::Rgb(170, 175, 180),    // #ABB0B6
                text_highlight: Color::Rgb(30, 32, 35),
                today_bg: Color::Rgb(250, 225, 190),
                selected_bg: Color::Rgb(225, 230, 235),
                tab_active_bg: Color::Rgb(242, 153, 58),
                tab_inactive_bg: Color::Rgb(228, 230, 232),
                button_bg: Color::Rgb(228, 230, 232),
                button_fg: Color::Rgb(92, 97, 102),
            },
        }
    }

    // Dynamic instance methods
    pub fn today_style(&self) -> Style {
        Style::default()
            .fg(if self.mode.is_light() { Color::Black } else { self.text_highlight })
            .bg(self.today_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_day_style(&self) -> Style {
        Style::default()
            .fg(if self.mode.is_light() { Color::Black } else { self.text_highlight })
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_tab_active(&self) -> Style {
        Style::default()
            .fg(if self.mode.is_light() { Color::White } else { self.text_highlight })
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
            .fg(if self.mode.is_light() { Color::Black } else { self.text_highlight })
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }
}
