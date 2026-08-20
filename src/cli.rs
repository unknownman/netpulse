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

    /// Generate shell completions for the specified shell (bash, elvish, fish, powershell, zsh)
    #[arg(long = "generate-completions", value_enum, value_name = "SHELL")]
    pub generate_completions: Option<clap_complete::Shell>,
}

fn has_no_color_env() -> bool {
    env::var("NO_COLOR")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::try_parse_from(["netpulse"]).unwrap();
        assert_eq!(cli.interval, 1000);
        assert_eq!(cli.interface, None);
        assert!(!cli.all);
        assert!(cli.generate_completions.is_none());
    }

    #[test]
    fn test_cli_custom_interval() {
        let cli = Cli::try_parse_from(["netpulse", "-i", "250", "--all"]).unwrap();
        assert_eq!(cli.interval, 250);
        assert!(cli.all);
    }

    #[test]
    fn test_cli_generate_completions() {
        let cli = Cli::try_parse_from(["netpulse", "--generate-completions", "zsh"]).unwrap();
        assert_eq!(cli.generate_completions, Some(clap_complete::Shell::Zsh));
    }
}
