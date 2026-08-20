use std::time::{Duration, Instant};

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::system_conf::read_system_conf;
use hickory_resolver::TokioAsyncResolver;
use tokio::sync::watch;

use crate::app::{DnsMetrics, DnsProbeResult};

const BENCHMARK_DOMAINS: &[&str] = &[
    "google.com",
    "cloudflare.com",
    "github.com",
    "wikipedia.org",
    "amazon.com",
];

pub async fn run_dns_collector(tx: watch::Sender<DnsMetrics>) {
    let (resolver, server_str) = match read_system_conf() {
        Ok((config, opts)) => {
            let ns_str = config
                .name_servers()
                .first()
                .map(|ns| ns.socket_addr.ip().to_string());
            let res = TokioAsyncResolver::tokio(config, opts);
            (res, ns_str)
        }
        Err(_) => {
            let res = TokioAsyncResolver::tokio(
                ResolverConfig::cloudflare(),
                ResolverOpts::default(),
            );
            (res, Some("1.1.1.1 (fallback)".into()))
        }
    };

    loop {
        let mut probe_futs = Vec::new();
        for &domain in BENCHMARK_DOMAINS {
            let res_clone = resolver.clone();
            probe_futs.push(async move {
                let start = Instant::now();
                match tokio::time::timeout(Duration::from_secs(2), res_clone.lookup_ip(domain)).await {
                    Ok(Ok(response)) => {
                        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
                        let first_ip = response.iter().next().map(|ip| ip.to_string());
                        DnsProbeResult {
                            domain: domain.to_string(),
                            latency_ms,
                            success: true,
                            resolved_ip: first_ip,
                            error: None,
                        }
                    }
                    Ok(Err(err)) => DnsProbeResult {
                        domain: domain.to_string(),
                        latency_ms: 0.0,
                        success: false,
                        resolved_ip: None,
                        error: Some(err.to_string()),
                    },
                    Err(_) => DnsProbeResult {
                        domain: domain.to_string(),
                        latency_ms: 0.0,
                        success: false,
                        resolved_ip: None,
                        error: Some("Timeout".to_string()),
                    },
                }
            });
        }

        let mut results = Vec::with_capacity(BENCHMARK_DOMAINS.len());
        for fut in probe_futs {
            results.push(fut.await);
        }

        let avg_latency_ms = compute_dns_avg(&results);

        let metrics = DnsMetrics {
            server: server_str.clone(),
            probes: results,
            avg_latency_ms,
            collected_at: Instant::now(),
        };

        if tx.send(metrics).is_err() {
            break;
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub fn compute_dns_avg(results: &[DnsProbeResult]) -> f64 {
    let successful: Vec<f64> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.latency_ms)
        .collect();

    if successful.is_empty() {
        0.0
    } else {
        successful.iter().sum::<f64>() / successful.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_dns_avg_all_success() {
        let results = vec![
            DnsProbeResult {
                domain: "google.com".into(),
                latency_ms: 10.0,
                success: true,
                resolved_ip: Some("142.250.190.46".into()),
                error: None,
            },
            DnsProbeResult {
                domain: "cloudflare.com".into(),
                latency_ms: 20.0,
                success: true,
                resolved_ip: Some("1.1.1.1".into()),
                error: None,
            },
        ];
        assert_eq!(compute_dns_avg(&results), 15.0);
    }

    #[test]
    fn test_compute_dns_avg_mixed() {
        let results = vec![
            DnsProbeResult {
                domain: "google.com".into(),
                latency_ms: 30.0,
                success: true,
                resolved_ip: Some("142.250.190.46".into()),
                error: None,
            },
            DnsProbeResult {
                domain: "fail.test".into(),
                latency_ms: 0.0,
                success: false,
                resolved_ip: None,
                error: Some("timeout".into()),
            },
        ];
        assert_eq!(compute_dns_avg(&results), 30.0);
    }

    #[test]
    fn test_compute_dns_avg_empty() {
        let results: Vec<DnsProbeResult> = vec![];
        assert_eq!(compute_dns_avg(&results), 0.0);
    }
}
