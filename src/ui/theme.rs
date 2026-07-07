use ratatui::style::Color;

/// Semantic color tokens — raw values chosen at theme creation time.
#[derive(Debug, Clone)]
pub struct Theme {
    // Core
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
    pub card_bg: Color,

    // Sidebar
    pub sidebar_bg: Color,
    pub sidebar_active: Color,
    pub sidebar_hover: Color,

    // Overlays (modal, command palette)
    pub overlay: Color,
    pub palette_bg: Color,
    pub palette_fg: Color,
    pub palette_highlight: Color,

    // Marketcap / volume colors
    pub large_cap: Color,
    pub mid_cap: Color,
    pub micro_cap: Color,
    pub nano_cap: Color,
    pub volume_high: Color,
    pub volume_low: Color,
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
            card_bg: Color::Rgb(22, 27, 34),

            sidebar_bg: Color::Rgb(18, 22, 28),
            sidebar_active: Color::Rgb(88, 166, 255),
            sidebar_hover: Color::Rgb(33, 38, 45),

            overlay: Color::Rgb(0, 0, 0),
            palette_bg: Color::Rgb(22, 27, 34),
            palette_fg: Color::Rgb(201, 209, 217),
            palette_highlight: Color::Rgb(48, 54, 61),

            large_cap: Color::Rgb(88, 166, 255),
            mid_cap: Color::Rgb(63, 185, 80),
            micro_cap: Color::Rgb(210, 153, 34),
            nano_cap: Color::Rgb(248, 81, 73),
            volume_high: Color::Rgb(63, 185, 80),
            volume_low: Color::Rgb(139, 148, 158),
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
            card_bg: Color::Rgb(20, 10, 40),

            sidebar_bg: Color::Rgb(15, 8, 30),
            sidebar_active: Color::Rgb(0, 255, 136),
            sidebar_hover: Color::Rgb(30, 15, 50),

            overlay: Color::Rgb(0, 0, 0),
            palette_bg: Color::Rgb(20, 10, 40),
            palette_fg: Color::Rgb(240, 240, 240),
            palette_highlight: Color::Rgb(60, 20, 100),

            large_cap: Color::Rgb(0, 200, 255),
            mid_cap: Color::Rgb(0, 255, 136),
            micro_cap: Color::Rgb(255, 200, 0),
            nano_cap: Color::Rgb(255, 50, 100),
            volume_high: Color::Rgb(0, 255, 100),
            volume_low: Color::Rgb(120, 100, 160),
        }
    }

    /// Terminal theme — deep black with amber/green accents like a Bloomberg terminal.
    pub fn terminal() -> Self {
        Self {
            bg: Color::Rgb(8, 8, 8),
            fg: Color::Rgb(200, 200, 180),
            accent: Color::Rgb(255, 190, 0),
            accent_dim: Color::Rgb(180, 130, 0),
            success: Color::Rgb(0, 200, 80),
            danger: Color::Rgb(255, 60, 60),
            warning: Color::Rgb(255, 140, 0),
            muted: Color::Rgb(100, 100, 90),
            border: Color::Rgb(40, 40, 35),
            highlight: Color::Rgb(50, 50, 40),
            card_bg: Color::Rgb(16, 16, 14),

            sidebar_bg: Color::Rgb(12, 12, 10),
            sidebar_active: Color::Rgb(255, 190, 0),
            sidebar_hover: Color::Rgb(24, 24, 20),

            overlay: Color::Rgb(0, 0, 0),
            palette_bg: Color::Rgb(16, 16, 14),
            palette_fg: Color::Rgb(200, 200, 180),
            palette_highlight: Color::Rgb(50, 50, 40),

            large_cap: Color::Rgb(0, 200, 255),
            mid_cap: Color::Rgb(0, 200, 80),
            micro_cap: Color::Rgb(255, 140, 0),
            nano_cap: Color::Rgb(255, 60, 60),
            volume_high: Color::Rgb(0, 200, 80),
            volume_low: Color::Rgb(100, 100, 90),
        }
    }

    /// Cyberpunk theme — pink/cyan neon on dark purple.
    pub fn cyberpunk() -> Self {
        Self {
            bg: Color::Rgb(15, 0, 25),
            fg: Color::Rgb(230, 230, 255),
            accent: Color::Rgb(0, 255, 255),
            accent_dim: Color::Rgb(0, 160, 160),
            success: Color::Rgb(0, 255, 128),
            danger: Color::Rgb(255, 0, 128),
            warning: Color::Rgb(255, 255, 0),
            muted: Color::Rgb(120, 80, 180),
            border: Color::Rgb(80, 0, 120),
            highlight: Color::Rgb(60, 0, 100),
            card_bg: Color::Rgb(20, 5, 35),

            sidebar_bg: Color::Rgb(18, 2, 30),
            sidebar_active: Color::Rgb(0, 255, 255),
            sidebar_hover: Color::Rgb(30, 10, 50),

            overlay: Color::Rgb(0, 0, 0),
            palette_bg: Color::Rgb(20, 5, 35),
            palette_fg: Color::Rgb(230, 230, 255),
            palette_highlight: Color::Rgb(60, 0, 100),

            large_cap: Color::Rgb(0, 200, 255),
            mid_cap: Color::Rgb(0, 255, 128),
            micro_cap: Color::Rgb(255, 255, 0),
            nano_cap: Color::Rgb(255, 0, 128),
            volume_high: Color::Rgb(0, 255, 128),
            volume_low: Color::Rgb(120, 80, 180),
        }
    }

    pub fn from_preset(preset: crate::data::models::ThemePreset) -> Self {
        match preset {
            crate::data::models::ThemePreset::Dark => Self::dark(),
            crate::data::models::ThemePreset::Degen => Self::degen(),
            crate::data::models::ThemePreset::Terminal => Self::terminal(),
            crate::data::models::ThemePreset::Cyberpunk => Self::cyberpunk(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
