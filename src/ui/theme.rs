use ratatui::style::Color;

/// Semantic color tokens — raw values chosen at theme creation time.
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub muted: Color,
    pub border: Color,
    pub highlight: Color,
    pub tab_active_bg: Color,
    pub tab_inactive_bg: Color,
    pub card_bg: Color,
}

impl Theme {
    /// Dark theme (OpenCode-inspired).
    pub fn dark() -> Self {
        Self {
            bg: Color::Rgb(13, 17, 23),
            fg: Color::Rgb(201, 209, 217),
            accent: Color::Rgb(88, 166, 255),
            accent_dim: Color::Rgb(31, 111, 235),
            success: Color::Rgb(63, 185, 80),
            danger: Color::Rgb(248, 81, 73),
            warning: Color::Rgb(210, 153, 34),
            muted: Color::Rgb(139, 148, 158),
            border: Color::Rgb(48, 54, 61),
            highlight: Color::Rgb(48, 54, 61),
            tab_active_bg: Color::Rgb(22, 27, 34),
            tab_inactive_bg: Color::Rgb(13, 17, 23),
            card_bg: Color::Rgb(22, 27, 34),
        }
    }

    /// Degen mode (vibrant, neon).
    pub fn degen() -> Self {
        Self {
            bg: Color::Rgb(10, 5, 20),
            fg: Color::Rgb(240, 240, 240),
            accent: Color::Rgb(0, 255, 136),
            accent_dim: Color::Rgb(0, 180, 90),
            success: Color::Rgb(0, 255, 100),
            danger: Color::Rgb(255, 50, 100),
            warning: Color::Rgb(255, 200, 0),
            muted: Color::Rgb(120, 100, 160),
            border: Color::Rgb(80, 50, 120),
            highlight: Color::Rgb(60, 20, 100),
            tab_active_bg: Color::Rgb(20, 10, 40),
            tab_inactive_bg: Color::Rgb(10, 5, 20),
            card_bg: Color::Rgb(20, 10, 40),
        }
    }

    pub fn from_preset(preset: crate::data::models::ThemePreset) -> Self {
        match preset {
            crate::data::models::ThemePreset::Dark => Self::dark(),
            crate::data::models::ThemePreset::Degen => Self::degen(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}