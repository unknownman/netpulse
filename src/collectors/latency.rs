use std::net::{IpAddr, TcpStream};
use std::time::{Duration, Instant};

use crate::app::{LatencyMetrics, LatencyStats, ProbeProtocol, ProbeResult};

struct ProbeTarget {
    addr: String,
    label: String,
}

async fn icmp_probe(addr: &str) -> Option<Duration> {
    let ip: IpAddr = addr.parse().ok()?;
    let payload = [0u8; 64];
    let (_, dur) = surge_ping::ping(ip, &payload).await.ok()?;
    Some(dur)
}

fn tcp_probe(addr: &str, port: u16) -> Option<Duration> {
    let sock_addr: std::net::SocketAddr = format!("{}:{}", addr, port).parse().ok()?;
    let start = Instant::now();
    TcpStream::connect_timeout(&sock_addr, Duration::from_secs(1)).ok()?;
    Some(start.elapsed())
}

async fn probe_target(target: &ProbeTarget) -> ProbeResult {
    if let Some(dur) = icmp_probe(&target.addr).await {
        return ProbeResult {
            target: target.label.clone(),
            protocol: ProbeProtocol::Icmp,
            latency_ms: dur.as_secs_f64() * 1000.0,
            success: true,
        };
    }

    for port in [443, 53] {
        if let Some(dur) = tcp_probe(&target.addr, port) {
            return ProbeResult {
                target: target.label.clone(),
                protocol: ProbeProtocol::Tcp,
                latency_ms: dur.as_secs_f64() * 1000.0,
                success: true,
            };
        }
    }

    ProbeResult {
        target: target.label.clone(),
        protocol: ProbeProtocol::Icmp,
        latency_ms: 0.0,
        success: false,
    }
}

fn compute_stats(results: &[ProbeResult]) -> LatencyStats {
    let successes: Vec<f64> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.latency_ms)
        .collect();

    let total = results.len() as f64;
    let loss = if total > 0.0 {
        ((total - successes.len() as f64) / total) * 100.0
    } else {
        100.0
    };

    if successes.is_empty() {
        return LatencyStats {
            min_ms: 0.0,
            avg_ms: 0.0,
            max_ms: 0.0,
            loss_pct: loss,
        };
    }

    let min = successes.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = successes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg = successes.iter().sum::<f64>() / successes.len() as f64;

    LatencyStats {
        min_ms: min,
        avg_ms: avg,
        max_ms: max,
        loss_pct: loss,
    }
}

pub async fn run_latency_collector(
    state: std::sync::Arc<std::sync::OnceLock<tokio::sync::watch::Sender<LatencyMetrics>>>,
    gateway: Option<String>,
) {
    let mut targets = vec![
        ProbeTarget {
            addr: "1.1.1.1".into(),
            label: "1.1.1.1".into(),
        },
        ProbeTarget {
            addr: "8.8.8.8".into(),
            label: "8.8.8.8".into(),
        },
    ];

    if let Some(ref gw) = gateway {
        targets.push(ProbeTarget {
            addr: gw.clone(),
            label: format!("gw:{}", gw),
        });
    }

    let mut history: Vec<ProbeResult> = Vec::new();
    let window = 10;

    loop {
        let mut results = Vec::new();
        for target in &targets {
            let result = probe_target(target).await;
            results.push(result);
        }

        history.extend(results.iter().cloned());
        if history.len() > window {
            history.drain(..history.len() - window);
        }

        let stats = compute_stats(&history);

        let metrics = LatencyMetrics {
            gateway: gateway.clone(),
            probes: results,
            stats,
        };

        if let Some(tx) = state.get() {
            if tx.send(metrics).is_err() {
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
