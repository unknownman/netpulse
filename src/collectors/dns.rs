use std::time::Instant;

use crate::app::DnsStatus;

pub struct DnsCollector {
    resolver: String,
}

pub struct DnsSample {
    pub resolution_ms: Option<f64>,
    pub status: DnsStatus,
}

impl DnsCollector {
    pub fn new(resolver: Option<String>) -> Self {
        Self {
            resolver: resolver.unwrap_or_else(|| "cloudflare.com".to_string()),
        }
    }

    pub fn sample(&self) -> DnsSample {
        let start = Instant::now();
        match dns_lookup::lookup_host(&self.resolver) {
            Ok(addrs) if !addrs.is_empty() => {
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                let status = if elapsed < 50.0 {
                    DnsStatus::Resolved
                } else if elapsed < 500.0 {
                    DnsStatus::Slow
                } else {
                    DnsStatus::Failed
                };
                DnsSample {
                    resolution_ms: Some(elapsed),
                    status,
                }
            }
            _ => DnsSample {
                resolution_ms: None,
                status: DnsStatus::Failed,
            },
        }
    }
}
