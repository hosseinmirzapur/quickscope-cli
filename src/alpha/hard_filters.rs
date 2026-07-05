use crate::data::models::*;

/// Check all hard filters against a FeatureVector and AlphaConfig.
/// Returns a HardFilterResult with pass/fail and individual failures.
pub fn check_hard_filters(fv: &FeatureVector, config: &AlphaConfig) -> HardFilterResult {
    let mut failures: Vec<FilterFailure> = Vec::new();

    // 1. Rug ratio
    if fv.rug_ratio > config.hf_rug_ratio_max {
        failures.push(FilterFailure {
            name: "rug_ratio".to_string(),
            value: fv.rug_ratio,
            threshold: config.hf_rug_ratio_max,
            direction: FilterDirection::Exceeded,
        });
    }

    // 2. Dev team hold rate
    if fv.dev_team_hold_rate > config.hf_dev_hold_max {
        failures.push(FilterFailure {
            name: "dev_team_hold_rate".to_string(),
            value: fv.dev_team_hold_rate,
            threshold: config.hf_dev_hold_max,
            direction: FilterDirection::Exceeded,
        });
    }

    // 3. Wash trading
    if config.hf_wash_trading && fv.is_wash_trading {
        failures.push(FilterFailure {
            name: "is_wash_trading".to_string(),
            value: 1.0,
            threshold: 0.0,
            direction: FilterDirection::Exceeded,
        });
    }

    // 4. Renounced mint (SOL: must be renounced)
    if config.hf_renounced_mint && !fv.renounced_mint {
        failures.push(FilterFailure {
            name: "renounced_mint".to_string(),
            value: if fv.renounced_mint { 1.0 } else { 0.0 },
            threshold: 1.0,
            direction: FilterDirection::Below,
        });
    }

    // 5. Liquidity minimum
    if fv.liquidity_usd < config.hf_liquidity_min_usd {
        failures.push(FilterFailure {
            name: "liquidity_usd".to_string(),
            value: fv.liquidity_usd,
            threshold: config.hf_liquidity_min_usd,
            direction: FilterDirection::Below,
        });
    }

    // 6. Creator hold + dev hold combo (extra check)
    if fv.creator_status == "creator_hold"
        && fv.dev_team_hold_rate > 0.10
    {
        failures.push(FilterFailure {
            name: "creator_hold_with_large_allocation".to_string(),
            value: fv.dev_team_hold_rate,
            threshold: 0.10,
            direction: FilterDirection::Exceeded,
        });
    }

    HardFilterResult {
        passed: failures.is_empty(),
        failures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_fv() -> FeatureVector {
        FeatureVector {
            rug_ratio: 0.05,
            dev_team_hold_rate: 0.03,
            is_wash_trading: false,
            renounced_mint: true,
            liquidity_usd: 50_000.0,
            creator_status: "creator_close".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_all_pass() {
        let config = AlphaConfig::default();
        let result = check_hard_filters(&good_fv(), &config);
        assert!(result.passed);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_fail_rug_ratio() {
        let mut fv = good_fv();
        fv.rug_ratio = 0.45;
        let result = check_hard_filters(&fv, &AlphaConfig::default());
        assert!(!result.passed);
        assert!(result.failures.iter().any(|f| f.name == "rug_ratio"));
    }

    #[test]
    fn test_fail_wash_trading() {
        let mut fv = good_fv();
        fv.is_wash_trading = true;
        let result = check_hard_filters(&fv, &AlphaConfig::default());
        assert!(!result.passed);
    }

    #[test]
    fn test_fail_low_liquidity() {
        let mut fv = good_fv();
        fv.liquidity_usd = 1_000.0;
        let result = check_hard_filters(&fv, &AlphaConfig::default());
        assert!(!result.passed);
    }

    #[test]
    fn test_config_can_relax() {
        let mut config = AlphaConfig::default();
        config.hf_rug_ratio_max = 0.50;
        config.hf_liquidity_min_usd = 1_000.0;

        let mut fv = good_fv();
        fv.rug_ratio = 0.45;
        fv.liquidity_usd = 2_000.0;

        let result = check_hard_filters(&fv, &config);
        assert!(result.passed);
    }
}