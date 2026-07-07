use crate::data::models::*;

/// Analyze a token for rug pull indicators.
/// Produces a RugReport with severity and detailed flags.
pub fn detect_rug(fv: &FeatureVector) -> RugReport {
    let mut flags: Vec<RugFlag> = Vec::new();

    // High rug ratio
    if fv.rug_ratio > 0.30 {
        flags.push(RugFlag {
            name: "high_rug_ratio".to_string(),
            severity: if fv.rug_ratio > 0.50 {
                RugSeverity::Critical
            } else {
                RugSeverity::High
            },
            detail: format!("Rug ratio is {:.1}% (threshold: 30%)", fv.rug_ratio * 100.0),
            value: fv.rug_ratio,
            threshold: 0.30,
        });
    } else if fv.rug_ratio > 0.20 {
        flags.push(RugFlag {
            name: "elevated_rug_ratio".to_string(),
            severity: RugSeverity::Medium,
            detail: format!("Rug ratio is {:.1}% (watch)", fv.rug_ratio * 100.0),
            value: fv.rug_ratio,
            threshold: 0.20,
        });
    }

    // High dev allocation
    if fv.dev_team_hold_rate > 0.05 {
        flags.push(RugFlag {
            name: "high_dev_allocation".to_string(),
            severity: if fv.dev_team_hold_rate > 0.15 {
                RugSeverity::High
            } else {
                RugSeverity::Medium
            },
            detail: format!(
                "Dev holds {:.1}% of supply (threshold: 5%)",
                fv.dev_team_hold_rate * 100.0
            ),
            value: fv.dev_team_hold_rate,
            threshold: 0.05,
        });
    }

    // Wash trading
    if fv.is_wash_trading {
        flags.push(RugFlag {
            name: "wash_trading_detected".to_string(),
            severity: RugSeverity::Critical,
            detail: "Wash trading detected — artificial volume".to_string(),
            value: 1.0,
            threshold: 0.0,
        });
    }

    // Mint not renounced (SOL-specific)
    if !fv.renounced_mint {
        flags.push(RugFlag {
            name: "mint_not_renounced".to_string(),
            severity: RugSeverity::Critical,
            detail: "Mint authority NOT renounced — more tokens can be created".to_string(),
            value: 0.0,
            threshold: 1.0,
        });
    }

    // Freeze not renounced
    if !fv.renounced_freeze {
        flags.push(RugFlag {
            name: "freeze_not_renounced".to_string(),
            severity: RugSeverity::High,
            detail: "Freeze authority NOT renounced — tokens can be frozen".to_string(),
            value: 0.0,
            threshold: 1.0,
        });
    }

    // Snipers present
    if fv.sniper_count > 5 {
        flags.push(RugFlag {
            name: "high_sniper_count".to_string(),
            severity: RugSeverity::Medium,
            detail: format!("{} snipers detected", fv.sniper_count),
            value: fv.sniper_count as f64,
            threshold: 5.0,
        });
    }

    // Creator still holding
    if fv.creator_status == "creator_hold" {
        flags.push(RugFlag {
            name: "creator_still_holding".to_string(),
            severity: RugSeverity::Medium,
            detail: "Creator still holds tokens".to_string(),
            value: 1.0,
            threshold: 0.0,
        });
    }

    // Compute overall severity
    let severity = if flags.iter().any(|f| f.severity == RugSeverity::Critical) {
        RugSeverity::Critical
    } else if flags.iter().any(|f| f.severity == RugSeverity::High) {
        RugSeverity::High
    } else if flags.iter().any(|f| f.severity == RugSeverity::Medium) {
        RugSeverity::Medium
    } else {
        RugSeverity::Low
    };

    let verdict = match severity {
        RugSeverity::Critical => "🛑 CRITICAL — DO NOT TRADE".to_string(),
        RugSeverity::High => "⚠️  HIGH RISK — Consider avoiding".to_string(),
        RugSeverity::Medium => "📋 MEDIUM — Proceed with caution".to_string(),
        RugSeverity::Low => "✅ LOW — No significant rug indicators".to_string(),
    };

    RugReport {
        severity,
        flags,
        verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_token_low_risk() {
        let fv = FeatureVector {
            rug_ratio: 0.05,
            dev_team_hold_rate: 0.03,
            is_wash_trading: false,
            renounced_mint: true,
            renounced_freeze: true,
            creator_status: "creator_close".to_string(),
            sniper_count: 0,
            ..Default::default()
        };
        let report = detect_rug(&fv);
        assert_eq!(report.severity, RugSeverity::Low);
        assert!(report.flags.is_empty());
    }

    #[test]
    fn test_critical_rug() {
        let fv = FeatureVector {
            rug_ratio: 0.55,
            is_wash_trading: true,
            renounced_mint: false,
            renounced_freeze: false,
            dev_team_hold_rate: 0.20,
            creator_status: "creator_hold".to_string(),
            sniper_count: 10,
            ..Default::default()
        };
        let report = detect_rug(&fv);
        assert_eq!(report.severity, RugSeverity::Critical);
        assert!(!report.flags.is_empty());
    }

    #[test]
    fn test_medium_rug() {
        let fv = FeatureVector {
            rug_ratio: 0.22,
            dev_team_hold_rate: 0.08,
            is_wash_trading: false,
            renounced_mint: true,
            renounced_freeze: true,
            creator_status: "creator_hold".to_string(),
            sniper_count: 3,
            ..Default::default()
        };
        let report = detect_rug(&fv);
        assert_eq!(report.severity, RugSeverity::Medium);
    }
}
