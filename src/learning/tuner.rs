use anyhow::Result;
use sqlx::SqlitePool;

use super::analyzer::{analyze_discrimination, FeatureDiscrimination};
use crate::data::models::AlphaConfig;
use crate::storage::{config, journal};

/// Auto-tuner: statistically adjusts alpha config weights and thresholds
/// based on discrimination analysis of winners vs losers.
///
/// Guard rails:
/// - ±5% max delta per run
/// - Min 10 wins AND 10 losses before first tune
/// - Each weight stays within [0.05, 0.40]
/// - Hard filter thresholds can tighten but never relax below safety floor
pub async fn run_auto_tune(pool: &SqlitePool) -> Result<Option<TuneResult>> {
    // 1. Load closed positions for analysis
    let closed = crate::storage::positions::get_closed_positions(pool, None, None).await?;

    // Separate winners and losers
    let winners: Vec<_> = closed
        .iter()
        .filter(|p| p.pnl_sol.unwrap_or(0.0) > 0.0)
        .collect();
    let losers: Vec<_> = closed
        .iter()
        .filter(|p| p.pnl_sol.unwrap_or(0.0) <= 0.0)
        .collect();

    // Guard: minimum sample size
    if winners.len() < 10 || losers.len() < 10 {
        tracing::info!(
            "Auto-tune skipped: need 10+ wins AND 10+ losses (have {}W/{}L)",
            winners.len(),
            losers.len()
        );
        return Ok(None);
    }

    // 2. Deserialize feature vectors
    let winner_fvs: Vec<_> = winners
        .iter()
        .filter_map(|p| {
            p.feature_vector.as_ref().and_then(|json| {
                serde_json::from_str::<crate::data::models::FeatureVector>(json).ok()
            })
        })
        .collect();
    let loser_fvs: Vec<_> = losers
        .iter()
        .filter_map(|p| {
            p.feature_vector.as_ref().and_then(|json| {
                serde_json::from_str::<crate::data::models::FeatureVector>(json).ok()
            })
        })
        .collect();

    if winner_fvs.is_empty() || loser_fvs.is_empty() {
        return Ok(None);
    }

    let winner_refs: Vec<&crate::data::models::FeatureVector> = winner_fvs.iter().collect();
    let loser_refs: Vec<&crate::data::models::FeatureVector> = loser_fvs.iter().collect();

    // 3. Discrimination analysis
    let discriminations = analyze_discrimination(&winner_refs, &loser_refs);

    // 4. Load current config
    let mut config = config::load_alpha_config(pool).await?;
    let old_config = config.clone();

    // 5. Nudge weights
    let max_delta = 0.05; // 5% max shift per run
    apply_weight_nudges(&mut config, &discriminations, max_delta);

    // 6. Nudge filter thresholds (tighten only)
    apply_filter_tighten(&mut config, &discriminations);

    // 7. Save new config
    config::save_alpha_config(pool, &config).await?;

    // 8. Log tuning history
    let old_weights = serde_json::to_string(&old_config)?;
    let new_weights = serde_json::to_string(&config)?;
    let discrim_json = serde_json::to_string(&discriminations)?;

    journal::log_tuning_run(
        pool,
        closed.len() as i64,
        winners.len() as i64,
        losers.len() as i64,
        &old_weights,
        &new_weights,
        &old_weights, // using same for old/new filters
        &new_weights,
        &discrim_json,
    )
    .await?;

    tracing::info!(
        "Auto-tune complete: {}W/{}L, nudged weights based on discrimination",
        winners.len(),
        losers.len()
    );

    Ok(Some(TuneResult {
        sample_size: closed.len(),
        wins: winners.len(),
        losses: losers.len(),
        discriminations,
    }))
}

#[derive(Debug, Clone)]
pub struct TuneResult {
    pub sample_size: usize,
    pub wins: usize,
    pub losses: usize,
    pub discriminations: Vec<FeatureDiscrimination>,
}

/// Nudge category weights based on feature discrimination.
/// Each weight stays within [0.05, 0.40], max ±5% total shift per run.
fn apply_weight_nudges(
    config: &mut AlphaConfig,
    discriminations: &[FeatureDiscrimination],
    max_delta: f64,
) {
    let category_map = build_category_map(discriminations);

    let nudges: Vec<(&mut f64, f64)> = vec![
        (
            &mut config.w_momentum,
            category_map.get("momentum").copied().unwrap_or(0.0),
        ),
        (
            &mut config.w_safety,
            category_map.get("safety").copied().unwrap_or(0.0),
        ),
        (
            &mut config.w_holder,
            category_map.get("holder").copied().unwrap_or(0.0),
        ),
        (
            &mut config.w_liquidity,
            category_map.get("liquidity").copied().unwrap_or(0.0),
        ),
        (
            &mut config.w_dev,
            category_map.get("dev").copied().unwrap_or(0.0),
        ),
        (
            &mut config.w_social,
            category_map.get("social").copied().unwrap_or(0.0),
        ),
    ];

    // Apply nudges with guard rails
    for (weight, nudge) in nudges {
        let clamped_nudge = nudge.clamp(-max_delta, max_delta);
        *weight = (*weight + clamped_nudge).clamp(0.05, 0.40);
    }
}

/// Build a map from category name → average discrimination of its features.
fn build_category_map(
    discriminations: &[FeatureDiscrimination],
) -> std::collections::HashMap<String, f64> {
    let mut map: std::collections::HashMap<String, Vec<f64>> = std::collections::HashMap::new();

    for d in discriminations {
        let category = if d.feature_name.starts_with("momentum") {
            "momentum"
        } else if d.feature_name.starts_with("safety") {
            "safety"
        } else if d.feature_name.starts_with("holder") {
            "holder"
        } else if d.feature_name.starts_with("liquidity") {
            "liquidity"
        } else if d.feature_name.starts_with("dev") {
            "dev"
        } else if d.feature_name.starts_with("social") || d.feature_name.starts_with("twitter") {
            "social"
        } else {
            continue;
        };

        let signal = if d.winner_mean > d.loser_mean {
            d.discrimination_power.min(3.0) / 3.0 * 0.03 // up to 3% nudge
        } else {
            -d.discrimination_power.min(3.0) / 3.0 * 0.03
        };

        map.entry(category.to_string()).or_default().push(signal);
    }

    map.into_iter()
        .map(|(k, v)| {
            let avg = v.iter().sum::<f64>() / v.len() as f64;
            (k, avg)
        })
        .collect()
}

/// Tighten hard filter thresholds if losers consistently fail them.
fn apply_filter_tighten(_config: &mut AlphaConfig, _discriminations: &[FeatureDiscrimination]) {
    // In v1, tightening is conservative:
    // - If rug_ratio discriminates strongly for losers (mean > 0.25), tighten by 0.02
    // - Never relax below safety floor
    // Full implementation deferred to post-MVP.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_map_building() {
        let disc = vec![
            FeatureDiscrimination {
                feature_name: "momentum_volume_1h".to_string(),
                winner_mean: 100_000.0,
                loser_mean: 10_000.0,
                winner_std: 10.0,
                loser_std: 10.0,
                discrimination_power: 3.0,
                recommended_direction: "increase".to_string(),
            },
            FeatureDiscrimination {
                feature_name: "safety_rug_ratio".to_string(),
                winner_mean: 0.05,
                loser_mean: 0.40,
                winner_std: 0.01,
                loser_std: 0.01,
                discrimination_power: 2.0,
                recommended_direction: "decrease".to_string(),
            },
        ];
        let map = build_category_map(&disc);
        assert!(map.contains_key("momentum"));
        assert!(map.contains_key("safety"));
        // momentum should have positive nudge (winners have higher volume)
        assert!(map["momentum"] > 0.0);
        // safety should have negative nudge (winners have lower rug ratio → inverse)
        // Actually, winners have lower rug ratio, so discrimination says "descrease" → negative nudge
    }
}
