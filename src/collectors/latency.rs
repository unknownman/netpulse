use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::app::LatencyStatus;

pub struct LatencyCollector {
    target: String,
}

pub struct LatencySample {
    pub latency_ms: Option<f64>,
    pub status: LatencyStatus,
}

impl LatencyCollector {
    pub fn new(target: Option<String>) -> Self {
        Self {
            target: target.unwrap_or_else(|| "1.1.1.1".to_string()),
        }
    }

    pub fn sample(&self) -> LatencySample {
        // TCP Connect Ping: try common ports
        let ports = [443, 53, 80];
        let host = self.target.clone();

        for port in ports {
            let addr = format!("{}:{}", host, port);
            let start = Instant::now();
            match TcpStream::connect_timeout(
                &addr.parse().unwrap_or_else(|_| {
                    format!("1.1.1.1:{}", port).parse().unwrap()
                }),
                Duration::from_secs(3),
            ) {
                Ok(_) => {
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    let status = if elapsed < 50.0 {
                        LatencyStatus::Good
                    } else if elapsed < 200.0 {
                        LatencyStatus::Degraded
                    } else {
                        LatencyStatus::Unreachable
                    };
                    return LatencySample {
                        latency_ms: Some(elapsed),
                        status,
                    };
                }
                Err(_) => continue,
            }
        }

        LatencySample {
            latency_ms: None,
            status: LatencyStatus::Unreachable,
        }
    }
}
