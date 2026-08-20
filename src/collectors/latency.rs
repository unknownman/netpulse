use std::net::IpAddr;
use std::time::{Duration, Instant};

use tokio::sync::watch;

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

async fn tcp_probe_port(addr: &str, port: u16) -> Option<Duration> {
    let sock_addr: std::net::SocketAddr = format!("{}:{}", addr, port).parse().ok()?;
    let start = Instant::now();
    match tokio::time::timeout(
        Duration::from_millis(1000),
        tokio::net::TcpStream::connect(sock_addr),
    )
    .await
    {
        Ok(Ok(_stream)) => Some(start.elapsed()),
        _ => None,
    }
}

async fn tcp_probe_concurrent(addr: &str) -> Option<Duration> {
    let probe_443 = tcp_probe_port(addr, 443);
    let probe_53 = tcp_probe_port(addr, 53);
    tokio::pin!(probe_443);
    tokio::pin!(probe_53);

    let mut p443_done = false;
    let mut p53_done = false;

    while !p443_done || !p53_done {
        tokio::select! {
            r = &mut probe_443, if !p443_done => {
                p443_done = true;
                if let Some(dur) = r {
                    return Some(dur);
                }
            }
            r = &mut probe_53, if !p53_done => {
                p53_done = true;
                if let Some(dur) = r {
                    return Some(dur);
                }
            }
        }
    }
    None
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

    if let Some(dur) = tcp_probe_concurrent(&target.addr).await {
        return ProbeResult {
            target: target.label.clone(),
            protocol: ProbeProtocol::Tcp,
            latency_ms: dur.as_secs_f64() * 1000.0,
            success: true,
        };
    }

    ProbeResult {
        target: target.label.clone(),
        protocol: ProbeProtocol::Icmp,
        latency_ms: 0.0,
        success: false,
    }
}

pub fn compute_stats(results: &[ProbeResult]) -> LatencyStats {
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
    tx: watch::Sender<LatencyMetrics>,
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
        let mut results = Vec::with_capacity(targets.len());
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

        if tx.send(metrics).is_err() {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stats_empty() {
        let stats = compute_stats(&[]);
        assert_eq!(stats.min_ms, 0.0);
        assert_eq!(stats.avg_ms, 0.0);
        assert_eq!(stats.max_ms, 0.0);
        assert_eq!(stats.loss_pct, 100.0);
    }

    #[test]
    fn test_compute_stats_all_success() {
        let probes = vec![
            ProbeResult {
                target: "1.1.1.1".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 10.0,
                success: true,
            },
            ProbeResult {
                target: "8.8.8.8".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 20.0,
                success: true,
            },
            ProbeResult {
                target: "gw".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 30.0,
                success: true,
            },
        ];
        let stats = compute_stats(&probes);
        assert_eq!(stats.min_ms, 10.0);
        assert_eq!(stats.avg_ms, 20.0);
        assert_eq!(stats.max_ms, 30.0);
        assert_eq!(stats.loss_pct, 0.0);
    }

    #[test]
    fn test_compute_stats_mixed_loss() {
        let probes = vec![
            ProbeResult {
                target: "1.1.1.1".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 15.0,
                success: true,
            },
            ProbeResult {
                target: "8.8.8.8".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 0.0,
                success: false,
            },
        ];
        let stats = compute_stats(&probes);
        assert_eq!(stats.min_ms, 15.0);
        assert_eq!(stats.avg_ms, 15.0);
        assert_eq!(stats.max_ms, 15.0);
        assert_eq!(stats.loss_pct, 50.0);
    }

    #[test]
    fn test_compute_stats_all_failure() {
        let probes = vec![
            ProbeResult {
                target: "1.1.1.1".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 0.0,
                success: false,
            },
            ProbeResult {
                target: "8.8.8.8".into(),
                protocol: ProbeProtocol::Icmp,
                latency_ms: 0.0,
                success: false,
            },
        ];
        let stats = compute_stats(&probes);
        assert_eq!(stats.min_ms, 0.0);
        assert_eq!(stats.avg_ms, 0.0);
        assert_eq!(stats.max_ms, 0.0);
        assert_eq!(stats.loss_pct, 100.0);
    }
}
