use ratatui::style::Color;

/// Format a market cap value to an abbreviated string with K/M/B suffix.
pub fn format_marketcap(mc: f64) -> String {
    if mc >= 1_000_000_000.0 {
        format!("${:.1}B", mc / 1_000_000_000.0)
    } else if mc >= 1_000_000.0 {
        format!("${:.1}M", mc / 1_000_000.0)
    } else if mc >= 1_000.0 {
        format!("${:.0}K", mc / 1_000.0)
    } else {
        format!("${:.0}", mc)
    }
}

/// Format a volume value to an abbreviated string.
pub fn format_volume(vol: f64) -> String {
    if vol >= 1_000_000.0 {
        format!("${:.1}M", vol / 1_000_000.0)
    } else if vol >= 1_000.0 {
        format!("${:.0}K", vol / 1_000.0)
    } else {
        format!("${:.0}", vol)
    }
}

/// Return the color for a given market cap value.
pub fn marketcap_color(mc: f64, theme: &super::theme::Theme) -> Color {
    if mc >= 10_000_000.0 {
        theme.large_cap
    } else if mc >= 1_000_000.0 {
        theme.mid_cap
    } else if mc >= 100_000.0 {
        theme.micro_cap
    } else {
        theme.nano_cap
    }
}

/// Return the color for a given volume value.
pub fn volume_color(vol: f64, theme: &super::theme::Theme) -> Color {
    if vol >= 500_000.0 {
        theme.volume_high
    } else {
        theme.volume_low
    }
}
