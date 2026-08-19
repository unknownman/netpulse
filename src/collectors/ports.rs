use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::app::PortStatus;

pub struct PortCollector {
    target: String,
    ports: Vec<(u16, String)>,
}

impl PortCollector {
    pub fn new(target: Option<String>) -> Self {
        Self {
            target: target.unwrap_or_else(|| "127.0.0.1".to_string()),
            ports: vec![
                (443, "HTTPS".into()),
                (80, "HTTP".into()),
                (53, "DNS".into()),
                (22, "SSH".into()),
            ],
        }
    }

    pub fn sample(&self) -> Vec<PortStatus> {
        self.ports
            .iter()
            .map(|(port, label)| {
                let addr = format!("{}:{}", self.target, port);
                let start = Instant::now();
                let open = TcpStream::connect_timeout(
                    &addr.parse().unwrap(),
                    Duration::from_secs(2),
                )
                .is_ok();
                let latency = if open {
                    Some(start.elapsed().as_secs_f64() * 1000.0)
                } else {
                    None
                };
                PortStatus {
                    port: *port,
                    label: label.clone(),
                    open,
                    latency_ms: latency,
                }
            })
            .collect()
    }
}
