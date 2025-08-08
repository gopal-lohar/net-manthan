use iced::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeOptions {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

// Implement conversion to Iced's Theme
impl From<ThemeOptions> for Theme {
    fn from(option: ThemeOptions) -> Self {
        match option {
            ThemeOptions::Light => Theme::Light,
            ThemeOptions::Dark => Theme::Dark,
            ThemeOptions::Dracula => Theme::Dracula,
            ThemeOptions::Nord => Theme::Nord,
            ThemeOptions::SolarizedLight => Theme::SolarizedLight,
            ThemeOptions::SolarizedDark => Theme::SolarizedDark,
            ThemeOptions::GruvboxLight => Theme::GruvboxLight,
            ThemeOptions::GruvboxDark => Theme::GruvboxDark,
            ThemeOptions::CatppuccinLatte => Theme::CatppuccinLatte,
            ThemeOptions::CatppuccinFrappe => Theme::CatppuccinFrappe,
            ThemeOptions::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            ThemeOptions::CatppuccinMocha => Theme::CatppuccinMocha,
            ThemeOptions::TokyoNight => Theme::TokyoNight,
            ThemeOptions::TokyoNightStorm => Theme::TokyoNightStorm,
            ThemeOptions::TokyoNightLight => Theme::TokyoNightLight,
            ThemeOptions::KanagawaWave => Theme::KanagawaWave,
            ThemeOptions::KanagawaDragon => Theme::KanagawaDragon,
            ThemeOptions::KanagawaLotus => Theme::KanagawaLotus,
            ThemeOptions::Moonfly => Theme::Moonfly,
            ThemeOptions::Nightfly => Theme::Nightfly,
            ThemeOptions::Oxocarbon => Theme::Oxocarbon,
            ThemeOptions::Ferra => Theme::Ferra,
        }
    }
}

// Implement display for each option
impl std::fmt::Display for ThemeOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ThemeOptions::Light => "Light Mode",
                ThemeOptions::Dark => "Dark Mode",
                ThemeOptions::Dracula => "Dracula",
                ThemeOptions::Nord => "Nord",
                ThemeOptions::SolarizedDark => "Solarized Dark",
                ThemeOptions::SolarizedLight => "Solarized Light",
                ThemeOptions::GruvboxDark => "Gruvbox Dark",
                ThemeOptions::GruvboxLight => "Gruvbox Light",
                ThemeOptions::CatppuccinLatte => "Catppuccin Latte",
                ThemeOptions::CatppuccinFrappe => "Catppuccin Frappe",
                ThemeOptions::CatppuccinMacchiato => "Catppuccin Macchiato",
                ThemeOptions::CatppuccinMocha => "Catppuccin Mocha",
                ThemeOptions::TokyoNight => "Tokyo Night",
                ThemeOptions::TokyoNightStorm => "Tokyo Night Storm",
                ThemeOptions::TokyoNightLight => "Tokyo Night Light",
                ThemeOptions::KanagawaWave => "Kanagawa Wave",
                ThemeOptions::KanagawaDragon => "Kanagawa Dragon",
                ThemeOptions::KanagawaLotus => "Kanagawa Lotus",
                ThemeOptions::Moonfly => "Moonfly",
                ThemeOptions::Nightfly => "Nightfly",
                ThemeOptions::Oxocarbon => "Oxocarbon",
                ThemeOptions::Ferra => "Ferra",
            }
        )
    }
}
