use crate::data::models::*;

/// Determine the trade mode based on Alpha Score and category sub-scores.
///
/// Rules (from design spec §6.5):
///
/// - EXPLODE: alpha >= 75 AND momentum >= 80 AND safety >= 70
/// - ALPHA:   alpha >= 55 AND safety >= 65
/// - SCALP:   momentum >= 70 AND (alpha < 55 OR safety < 65)
/// - FALLBACK: everything else
pub fn select_mode(alpha: f64, scores: &CategoryScores) -> TradeMode {
    // Normalize sub-scores to 0-100 scale
    let momentum = scores.momentum * 100.0;
    let safety = scores.safety * 100.0;

    // EXPLODE
    if alpha >= 75.0 && momentum >= 80.0 && safety >= 70.0 {
        return TradeMode::Explode;
    }

    // ALPHA
    if alpha >= 55.0 && safety >= 65.0 {
        return TradeMode::Alpha;
    }

    // SCALP
    if momentum >= 70.0 && (alpha < 55.0 || safety < 65.0) {
        return TradeMode::Scalp;
    }

    // FALLBACK
    TradeMode::Fallback
}

/// Return sizing bounds (min/max SOL) for a given trade mode.
pub fn sizing_for_mode(mode: &TradeMode) -> SizingBounds {
    match mode {
        TradeMode::Explode => SizingBounds {
            min_sol: 0.5,
            max_sol: 1.0,
        },
        TradeMode::Alpha => SizingBounds {
            min_sol: 0.2,
            max_sol: 0.5,
        },
        TradeMode::Scalp => SizingBounds {
            min_sol: 0.1,
            max_sol: 0.2,
        },
        TradeMode::Fallback => SizingBounds {
            min_sol: 0.05,
            max_sol: 0.1,
        },
    }
}

/// Return default TP/SL percentages for a mode.
pub fn exit_params_for_mode(mode: &TradeMode) -> (f64, f64) {
    match mode {
        TradeMode::Explode => (200.0, 60.0), // TP +200%, SL -60%
        TradeMode::Alpha => (100.0, 40.0),   // TP +100%, SL -40%
        TradeMode::Scalp => (20.0, 15.0),    // TP +20%, SL -15%
        TradeMode::Fallback => (10.0, 20.0), // TP +10%, SL -20%
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores(m: f64, s: f64, h: f64, l: f64, d: f64, soc: f64) -> CategoryScores {
        CategoryScores {
            momentum: m / 100.0,
            safety: s / 100.0,
            holder_quality: h / 100.0,
            liquidity: l / 100.0,
            dev_trust: d / 100.0,
            social: soc / 100.0,
        }
    }

    #[test]
    fn test_explode_mode() {
        // Need very high scores to push alpha >= 75
        let s = scores(95.0, 85.0, 90.0, 90.0, 80.0, 80.0);
        let config = AlphaConfig::default();
        let alpha = crate::alpha::scoring::alpha_score(&s, &config);
        let mode = select_mode(alpha, &s);
        assert_eq!(mode, TradeMode::Explode);
    }

    #[test]
    fn test_alpha_mode() {
        // Alpha >= 55, safety >= 65
        let s = scores(60.0, 70.0, 65.0, 60.0, 50.0, 50.0);
        let config = AlphaConfig::default();
        let alpha = crate::alpha::scoring::alpha_score(&s, &config);
        assert!(alpha >= 55.0, "alpha={}", alpha);
        let mode = select_mode(alpha, &s);
        assert_eq!(mode, TradeMode::Alpha);
    }

    #[test]
    fn test_scalp_mode() {
        let s = scores(75.0, 40.0, 50.0, 30.0, 20.0, 10.0);
        // momentum >= 70, safety < 65 → SCALP
        let config = AlphaConfig::default();
        let alpha = crate::alpha::scoring::alpha_score(&s, &config);
        let mode = select_mode(alpha, &s);
        assert_eq!(mode, TradeMode::Scalp);
    }

    #[test]
    fn test_fallback_mode() {
        let s = scores(30.0, 30.0, 20.0, 10.0, 10.0, 5.0);
        let config = AlphaConfig::default();
        let alpha = crate::alpha::scoring::alpha_score(&s, &config);
        let mode = select_mode(alpha, &s);
        assert_eq!(mode, TradeMode::Fallback);
    }

    #[test]
    fn test_sizing_bounds() {
        assert_eq!(sizing_for_mode(&TradeMode::Explode).max_sol, 1.0);
        assert_eq!(sizing_for_mode(&TradeMode::Fallback).min_sol, 0.05);
    }

    #[test]
    fn test_exit_params() {
        let (tp, sl) = exit_params_for_mode(&TradeMode::Explode);
        assert_eq!(tp, 200.0);
        assert_eq!(sl, 60.0);

        let (tp, sl) = exit_params_for_mode(&TradeMode::Scalp);
        assert_eq!(tp, 20.0);
        assert_eq!(sl, 15.0);
    }
}
