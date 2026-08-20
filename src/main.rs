mod app;
mod cli;
mod collectors;
mod ui;
mod utils;

use std::io;
use std::sync::{Arc, OnceLock};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::watch;

use app::{LatencyMetrics, LatencyStats, NetworkSnapshot};
use cli::Cli;
use collectors::bandwidth::run_bandwidth_collector;
use collectors::latency::run_latency_collector;

static LATENCY_TX: OnceLock<watch::Sender<LatencyMetrics>> = OnceLock::new();

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let gw = utils::gateway::detect_gateway();
    let gw_str = gw.map(|ip| ip.to_string());

    let initial_snap = NetworkSnapshot {
        timestamp: std::time::Instant::now(),
        interfaces: Vec::new(),
    };

    let initial_latency = LatencyMetrics {
        gateway: gw_str.clone(),
        probes: Vec::new(),
        stats: LatencyStats {
            min_ms: 0.0,
            avg_ms: 0.0,
            max_ms: 0.0,
            loss_pct: 100.0,
        },
    };

    let (snap_tx, snap_rx) = watch::channel(initial_snap);
    let (lat_tx, mut lat_rx) = watch::channel(initial_latency);
    LATENCY_TX.set(lat_tx).ok();

    let lat_state = Arc::new(OnceLock::<watch::Sender<LatencyMetrics>>::new());
    lat_state.set(LATENCY_TX.get().unwrap().clone()).ok();

    tokio::spawn(run_bandwidth_collector(snap_tx, cli.clone()));
    tokio::spawn(run_latency_collector(lat_state, gw_str));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = std::time::Duration::from_millis(66);
    let result = run_app(&mut terminal, &snap_rx, &mut lat_rx, &cli, tick_rate).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    snap_rx: &watch::Receiver<NetworkSnapshot>,
    lat_rx: &mut watch::Receiver<LatencyMetrics>,
    cli: &Cli,
    tick_rate: std::time::Duration,
) -> anyhow::Result<()> {
    loop {
        {
            let snap = snap_rx.borrow().clone();
            let latency = lat_rx.borrow_and_update().clone();
            terminal.draw(|f| ui::dashboard::render(f, &snap, &latency, cli.no_color))?;
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
