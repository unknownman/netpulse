mod app;
mod cli;
mod collectors;
mod ui;
mod utils;

use std::io;

use clap::{CommandFactory, Parser};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::watch;

use app::{DnsMetrics, LatencyMetrics, LatencyStats, NetworkSnapshot, PortsMetrics};
use cli::Cli;
use collectors::bandwidth::run_bandwidth_collector;
use collectors::dns::run_dns_collector;
use collectors::latency::run_latency_collector;
use collectors::ports::run_ports_collector;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle shell completions generation if requested
    if let Some(shell) = cli.generate_completions {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "netpulse", &mut io::stdout());
        return;
    }

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

    let initial_dns = DnsMetrics {
        server: None,
        probes: Vec::new(),
        avg_latency_ms: 0.0,
        collected_at: std::time::Instant::now(),
    };

    let initial_ports = PortsMetrics {
        listening: Vec::new(),
        collected_at: std::time::Instant::now(),
    };

    let (snap_tx, snap_rx) = watch::channel(initial_snap);
    let (lat_tx, mut lat_rx) = watch::channel(initial_latency);
    let (dns_tx, mut dns_rx) = watch::channel(initial_dns);
    let (ports_tx, mut ports_rx) = watch::channel(initial_ports);

    tokio::spawn(run_bandwidth_collector(snap_tx, cli.clone()));
    tokio::spawn(run_latency_collector(lat_tx, gw_str));
    tokio::spawn(run_dns_collector(dns_tx));
    tokio::spawn(run_ports_collector(ports_tx));

    // Install panic hook to ensure terminal restoration even on crash
    let original_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_panic(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = std::time::Duration::from_millis(66);
    let result = run_app(
        &mut terminal,
        &snap_rx,
        &mut lat_rx,
        &mut dns_rx,
        &mut ports_rx,
        &cli,
        tick_rate,
    )
    .await;

    // Clean up terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    snap_rx: &watch::Receiver<NetworkSnapshot>,
    lat_rx: &mut watch::Receiver<LatencyMetrics>,
    dns_rx: &mut watch::Receiver<DnsMetrics>,
    ports_rx: &mut watch::Receiver<PortsMetrics>,
    cli: &Cli,
    tick_rate: std::time::Duration,
) -> anyhow::Result<()> {
    loop {
        {
            let snap = snap_rx.borrow();
            let latency = lat_rx.borrow_and_update();
            let dns = dns_rx.borrow_and_update();
            let ports = ports_rx.borrow_and_update();
            terminal.draw(|f| {
                ui::dashboard::render(f, &snap, &latency, &dns, &ports, cli.no_color)
            })?;
        }

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
                        }
                        _ => {}
                    }
                }
                Event::Resize(..) => {
                    // Force terminal re-draw cleanly on resize
                    terminal.autoresize()?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
