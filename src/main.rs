mod app;
mod cli;
mod collectors;
mod ui;
mod utils;

use std::io;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::watch;

use cli::Cli;
use collectors::{bandwidth::BandwidthCollector, dns::DnsCollector, latency::LatencyCollector, ports::PortCollector};
use app::{AppState, LatencyStatus, NetworkSnapshot};
use utils::interface::{detect_active_interface, get_default_gateway};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let iface = match &cli.interface {
        Some(i) => i.clone(),
        None => detect_active_interface()?,
    };

    let gw = get_default_gateway().map(|ip| ip.to_string());

    if cli.check {
        return run_check(&iface, gw.as_deref());
    }

    run_tui(&iface, gw.as_deref(), cli.interval).await
}

fn run_check(iface: &str, gw: Option<&str>) -> anyhow::Result<()> {
    println!("netpulse --check: one-shot health report\n");

    let mut bw = BandwidthCollector::new();
    let lat = LatencyCollector::new(Some("1.1.1.1".into()));
    let dns = DnsCollector::new(None);
    let port_collector = PortCollector::new(Some("127.0.0.1".into()));

    // Bandwidth
    let bw_sample = bw.sample(iface)?;
    println!("Interface : {}", iface);
    println!("Gateway   : {}", gw.unwrap_or("N/A"));
    println!(
        "Bandwidth : TX {:.1} kbit/s  RX {:.1} kbit/s",
        bw_sample.bps_tx / 1000.0,
        bw_sample.bps_rx / 1000.0
    );

    // Latency
    let lat_sample = lat.sample();
    match lat_sample.latency_ms {
        Some(ms) => println!("Latency   : {:.1}ms  [{}]", ms, fmt_status(&lat_sample.status)),
        None => println!("Latency   : unreachable"),
    }

    // DNS
    let dns_sample = dns.sample();
    match dns_sample.resolution_ms {
        Some(ms) => println!("DNS       : {:.1}ms  [{}]", ms, fmt_dns(&dns_sample.status)),
        None => println!("DNS       : failed"),
    }

    // Ports
    let ports = port_collector.sample();
    println!("\nPort scan:");
    for p in &ports {
        let icon = if p.open { "●" } else { "○" };
        let lat = p.latency_ms.map(|v| format!("{:.1}ms", v)).unwrap_or_else(|| "-".into());
        println!("  {} {:<5}:{:<5} {}", icon, p.label, p.port, lat);
    }

    // Determine exit code
    let healthy = lat_sample.status != LatencyStatus::Unreachable
        && dns_sample.status != crate::app::DnsStatus::Failed;

    println!("\n{}", if healthy { "✓ Healthy" } else { "✗ Degraded" });

    // Exit code: 0 = healthy, 1 = degraded
    if !healthy {
        std::process::exit(1);
    }
    Ok(())
}

async fn run_tui(iface: &str, gw: Option<&str>, interval_ms: u64) -> anyhow::Result<()> {
    // Initialize collectors
    let mut bw_collector = BandwidthCollector::new();
    let lat_collector = LatencyCollector::new(Some("1.1.1.1".into()));
    let dns_collector = DnsCollector::new(None);
    let port_collector = PortCollector::new(Some("127.0.0.1".into()));

    // Initial sample
    let bw = bw_collector.sample(iface)?;
    let lat = lat_collector.sample();
    let dns = dns_collector.sample();
    let ports = port_collector.sample();

    let snapshot = NetworkSnapshot {
        timestamp: Instant::now(),
        bytes_tx: bw.total_tx,
        bytes_rx: bw.total_rx,
        bandwidth_tx_bps: bw.bps_tx,
        bandwidth_rx_bps: bw.bps_rx,
        latency_ms: lat.latency_ms,
        latency_status: lat.status,
        dns_resolution_ms: dns.resolution_ms,
        dns_status: dns.status,
        ports,
        interface_name: iface.to_string(),
        gateway: gw.map(|s| s.to_string()),
    };

    let (tx, rx) = watch::channel(snapshot.clone());

    // Spawn collector task
    let iface_clone = iface.to_string();
    let gw_owned = gw.map(|s| s.to_string());
    tokio::spawn(async move {
        let mut bw_col = BandwidthCollector::new();
        let lat_col = LatencyCollector::new(Some("1.1.1.1".into()));
        let dns_col = DnsCollector::new(None);
        let port_col = PortCollector::new(Some("127.0.0.1".into()));

        loop {
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;

            let bw = bw_col.sample(&iface_clone).unwrap_or({
                collectors::bandwidth::BandwidthSample {
                    total_tx: 0,
                    total_rx: 0,
                    bps_tx: 0.0,
                    bps_rx: 0.0,
                }
            });
            let lat = lat_col.sample();
            let dns = dns_col.sample();
            let ports = port_col.sample();

            let snapshot = NetworkSnapshot {
                timestamp: Instant::now(),
                bytes_tx: bw.total_tx,
                bytes_rx: bw.total_rx,
                bandwidth_tx_bps: bw.bps_tx,
                bandwidth_rx_bps: bw.bps_rx,
                latency_ms: lat.latency_ms,
                latency_status: lat.status,
                dns_resolution_ms: dns.resolution_ms,
                dns_status: dns.status,
                ports,
                interface_name: iface_clone.clone(),
                gateway: gw_owned.clone(),
            };

            let _ = tx.send(snapshot);
        }
    });

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(snapshot);
    let tick_rate = Duration::from_millis(100); // ~10 FPS

    loop {
        // Snapshot state (non-blocking borrow)
        {
            let snap = rx.borrow().clone();
            state.update(snap);
        }

        // Draw
        terminal.draw(|f| ui::dashboard::draw(f, &state))?;

        // Handle input
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn fmt_status(s: &LatencyStatus) -> &'static str {
    match s {
        LatencyStatus::Good => "OK",
        LatencyStatus::Degraded => "WARN",
        LatencyStatus::Unreachable => "FAIL",
        LatencyStatus::Unknown => "----",
    }
}

fn fmt_dns(s: &crate::app::DnsStatus) -> &'static str {
    match s {
        crate::app::DnsStatus::Resolved => "OK",
        crate::app::DnsStatus::Slow => "WARN",
        crate::app::DnsStatus::Failed => "FAIL",
        crate::app::DnsStatus::Unknown => "----",
    }
}
