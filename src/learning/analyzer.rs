use crate::data::models::*;

/// Statistical discrimination analysis: "what do winners have in common that losers don't?"
///
/// For each feature, computes:
/// - winner_mean, loser_mean
/// - discrimination_power = |winner_mean - loser_mean| / (pooled_std + epsilon)
/// - recommended_direction: "increase" or "decrease"
///
/// Higher discrimination → feature is better at separating winners from losers.
pub fn analyze_discrimination(
    winners: &[&FeatureVector],
    losers: &[&FeatureVector],
) -> Vec<FeatureDiscrimination> {
    if winners.is_empty() || losers.is_empty() {
        return Vec::new();
    }

    type FeatureExtractor = (&'static str, fn(&FeatureVector) -> f64);
    let features: Vec<FeatureExtractor> = vec![
        ("momentum_volume_1h", |fv| fv.volume_1h.unwrap_or(0.0)),
        ("momentum_swaps_1h", |fv| fv.swaps_1h.unwrap_or(0) as f64),
        ("momentum_hot_level", |fv| fv.hot_level.unwrap_or(0) as f64),
        ("momentum_change_1h", |fv| fv.price_change_1h.unwrap_or(0.0)),
        ("safety_rug_ratio", |fv| fv.rug_ratio),
        ("safety_top_10_holder", |fv| fv.top_10_holder_rate),
        ("holder_smart_wallets", |fv| fv.smart_degen_count as f64),
        ("holder_renowned", |fv| fv.renowned_count as f64),
        ("holder_dev_hold", |fv| fv.dev_team_hold_rate),
        ("liquidity_usd", |fv| fv.liquidity_usd),
        ("dev_cto_flag", |fv| if fv.cto_flag { 1.0 } else { 0.0 }),
        (
            "dev_dexscr_boost",
            |fv| if fv.dexscr_boost { 1.0 } else { 0.0 },
        ),
        ("social_mentions", |fv| {
            fv.twitter_mentions_1h.unwrap_or(0) as f64
        }),
        ("social_sentiment", |fv| fv.twitter_sentiment.unwrap_or(0.5)),
    ];

    features
        .iter()
        .map(|(name, getter)| {
            let getter = *getter;
            let w_mean = mean_of(winners, getter);
            let l_mean = mean_of(losers, getter);
            let w_var = variance_of(winners, getter, w_mean);
            let l_var = variance_of(losers, getter, l_mean);
            let pooled_std = ((w_var + l_var) / 2.0).sqrt().max(1e-6);
            let discrimination = (w_mean - l_mean).abs() / pooled_std;

            FeatureDiscrimination {
                feature_name: name.to_string(),
                winner_mean: w_mean,
                loser_mean: l_mean,
                winner_std: w_var.sqrt(),
                loser_std: l_var.sqrt(),
                discrimination_power: discrimination,
                recommended_direction: if w_mean > l_mean {
                    "increase".to_string()
                } else {
                    "decrease".to_string()
                },
            }
        })
        .collect()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeatureDiscrimination {
    pub feature_name: String,
    pub winner_mean: f64,
    pub loser_mean: f64,
    pub winner_std: f64,
    pub loser_std: f64,
    pub discrimination_power: f64,
    pub recommended_direction: String, // "increase" or "decrease"
}

fn mean_of(fvs: &[&FeatureVector], getter: fn(&FeatureVector) -> f64) -> f64 {
    if fvs.is_empty() {
        return 0.0;
    }
    fvs.iter().map(|fv| getter(fv)).sum::<f64>() / fvs.len() as f64
}

fn variance_of(fvs: &[&FeatureVector], getter: fn(&FeatureVector) -> f64, mean: f64) -> f64 {
    if fvs.len() <= 1 {
        return 0.0;
    }
    fvs.iter()
        .map(|fv| {
            let diff = getter(fv) - mean;
            diff * diff
        })
        .sum::<f64>()
        / (fvs.len() - 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fv(vol: f64, smart: u64, rug: f64) -> FeatureVector {
        FeatureVector {
            volume_1h: Some(vol),
            smart_degen_count: smart,
            rug_ratio: rug,
            ..Default::default()
        }
    }

    #[test]
    fn test_discrimination_clear_separation() {
        let winners = vec![
            make_fv(100_000.0, 20, 0.03),
            make_fv(120_000.0, 25, 0.02),
            make_fv(90_000.0, 18, 0.04),
        ];
        let winners_refs: Vec<&FeatureVector> = winners.iter().collect();

        let losers = vec![
            make_fv(10_000.0, 2, 0.45),
            make_fv(5_000.0, 1, 0.55),
            make_fv(15_000.0, 3, 0.40),
        ];
        let losers_refs: Vec<&FeatureVector> = losers.iter().collect();

        let results = analyze_discrimination(&winners_refs, &losers_refs);
        assert!(!results.is_empty());

        // Volume should strongly discriminate
        let vol = results
            .iter()
            .find(|r| r.feature_name == "momentum_volume_1h")
            .unwrap();
        assert!(vol.winner_mean > vol.loser_mean);
        assert!(vol.discrimination_power > 1.0);

        // Rug ratio: winners should have lower rug ratio
        let rug = results
            .iter()
            .find(|r| r.feature_name == "safety_rug_ratio")
            .unwrap();
        assert!(rug.winner_mean < rug.loser_mean);
    }

    #[test]
    fn test_empty_inputs() {
        let empty: Vec<&FeatureVector> = vec![];
        let results = analyze_discrimination(&empty, &empty);
        assert!(results.is_empty());
    }
}
