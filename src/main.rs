use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "quickscope", about = "Solana memecoin alpha hunting TUI")]
struct Cli {
    /// Path to config file
    #[arg(short, long, env = "QUICKSCOPE_CONFIG")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let _cli = Cli::parse();
    println!("⚡ QuickScope v0.1.0 — Solana memecoin alpha hunting");
    Ok(())
}
