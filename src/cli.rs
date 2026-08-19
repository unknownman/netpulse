use std::env;

use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(
    name = "netpulse",
    version,
    about = "Ultra-lightweight, zero-flicker network pulse tool"
)]
pub struct Cli {
    /// Refresh interval in milliseconds
    #[arg(short = 'i', long, default_value = "1000")]
    pub interval: u64,

    /// Network interface to monitor (auto-detects if omitted)
    #[arg(long)]
    pub interface: Option<String>,

    /// Show all interfaces, including inactive ones
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Disable colored output (also respects NO_COLOR env var)
    #[arg(long, default_value_t = has_no_color_env())]
    pub no_color: bool,
}

fn has_no_color_env() -> bool {
    env::var("NO_COLOR")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
}
