use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "netpulse", version, about = "Ultra-lightweight, zero-flicker network pulse tool")]
pub struct Cli {
    /// Interface to monitor (auto-detects if omitted)
    #[arg(short, long)]
    pub interface: Option<String>,

    /// Refresh interval in milliseconds
    #[arg(short = 'd', long, default_value = "1000")]
    pub interval: u64,

    /// Run a one-shot health check and exit
    #[arg(short, long)]
    pub check: bool,
}
