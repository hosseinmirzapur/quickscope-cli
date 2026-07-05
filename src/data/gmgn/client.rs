use anyhow::{Context, Result};
use serde_json::Value;

/// GMGN v1 read-only client — delegates all API calls to `gmgn-cli`.
///
/// The official GMGN API requires Ed25519 request signing, which is handled
/// automatically by `gmgn-cli`. QuickScope calls the CLI as a subprocess
/// and parses its JSON output.
///
/// Prerequisites:
///   - `gmgn-cli` installed globally (`npm install -g gmgn-cli`)
///   - `GMGN_API_KEY` configured via `gmgn-cli config --apply <KEY>`
pub struct GmgnClient;

impl GmgnClient {
    /// Run a gmgn-cli command and return the parsed JSON output.
    async fn run(args: &[&str]) -> Result<Value> {
        let output = tokio::process::Command::new("gmgn-cli")
            .args(args)
            .arg("--raw")
            .output()
            .await
            .with_context(|| format!("failed to execute gmgn-cli with args: {:?}", args))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gmgn-cli error (exit {}): {}", output.status, stderr.trim());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let value: Value = serde_json::from_str(&stdout)
            .with_context(|| format!("parsing gmgn-cli output: {}", &stdout[..stdout.len().min(200)]))?;

        if let Some(code) = value.get("code").and_then(|c| c.as_i64()) {
            if code != 0 {
                let msg = value.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
                anyhow::bail!("GMGN API error (code {}): {}", code, msg);
            }
        }

        Ok(value.get("data").cloned().unwrap_or(value))
    }

    // ── Market Endpoints ────────────────────────────────────────

    /// Trending tokens. Returns the `data.rank` array.
    pub async fn trending(&self, interval: &str, limit: u32, order_by: &str) -> Result<Value> {
        let data = Self::run(&[
            "market", "trending",
            "--chain", "sol",
            "--interval", interval,
            "--limit", &limit.to_string(),
            "--order-by", order_by,
            "--direction", "desc",
        ]).await?;
        let rank = data.get("rank")
            .context("missing data.rank in trending response")?
            .clone();
        Ok(rank)
    }

    /// Kline candles. Returns `data.list` array.
    pub async fn kline(&self, address: &str, resolution: &str, from: i64, to: i64) -> Result<Value> {
        let data = Self::run(&[
            "market", "kline",
            "--chain", "sol",
            "--address", address,
            "--resolution", resolution,
            "--from", &from.to_string(),
            "--to", &to.to_string(),
        ]).await?;
        let list = data.get("list")
            .context("missing data.list in kline response")?
            .clone();
        Ok(list)
    }

    /// Trenches (newly launched tokens). Returns the full data object.
    pub async fn trenches(&self, token_type: &str) -> Result<Value> {
        Self::run(&[
            "market", "trenches",
            "--chain", "sol",
            "--type", token_type,
        ]).await
    }

    /// Token signals.
    pub async fn signal(&self) -> Result<Value> {
        Self::run(&["market", "signal", "--chain", "sol"]).await
    }

    // ── Token Endpoints ────────────────────────────────────────

    /// Token info.
    pub async fn token_info(&self, address: &str) -> Result<Value> {
        Self::run(&["token", "info", "--chain", "sol", "--address", address]).await
    }

    /// Token security.
    pub async fn token_security(&self, address: &str) -> Result<Value> {
        Self::run(&["token", "security", "--chain", "sol", "--address", address]).await
    }

    /// Token holders by wallet tag.
    pub async fn token_holders(&self, address: &str, tag: &str, limit: u32) -> Result<Value> {
        Self::run(&[
            "token", "holders",
            "--chain", "sol",
            "--address", address,
            "--tag", tag,
            "--limit", &limit.to_string(),
        ]).await
    }

    // ── Portfolio Endpoints ────────────────────────────────────

    pub async fn portfolio_info(&self, address: &str) -> Result<Value> {
        Self::run(&["portfolio", "info", "--chain", "sol", "--address", address]).await
    }

    pub async fn portfolio_holdings(&self, address: &str) -> Result<Value> {
        Self::run(&["portfolio", "holdings", "--chain", "sol", "--address", address]).await
    }

    // ── Track Endpoints ─────────────────────────────────────────

    /// Smart money trades.
    pub async fn smartmoney(&self, limit: u32) -> Result<Value> {
        Self::run(&["track", "smartmoney", "--chain", "sol", "--limit", &limit.to_string()]).await
    }

    /// KOL trades.
    pub async fn kol_trades(&self, limit: u32) -> Result<Value> {
        Self::run(&["track", "kol", "--chain", "sol", "--limit", &limit.to_string()]).await
    }

    // ── Quote ────────────────────────────────────────────────────

    /// Quote for paper pricing.
    pub async fn quote(&self, input_token: &str, output_token: &str, amount: f64) -> Result<Value> {
        Self::run(&[
            "swap", "quote",
            "--chain", "sol",
            "--input-token", input_token,
            "--output-token", output_token,
            "--input-amount", &amount.to_string(),
        ]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = GmgnClient;
    }
}