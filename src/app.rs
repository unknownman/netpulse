use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct NetworkSnapshot {
    #[allow(dead_code)]
    pub timestamp: Instant,

    // Bandwidth
    #[allow(dead_code)]
    pub bytes_tx: u64,
    #[allow(dead_code)]
    pub bytes_rx: u64,
    pub bandwidth_tx_bps: f64,
    pub bandwidth_rx_bps: f64,

    // Latency
    pub latency_ms: Option<f64>,
    pub latency_status: LatencyStatus,

    // DNS
    pub dns_resolution_ms: Option<f64>,
    pub dns_status: DnsStatus,

    // Ports
    pub ports: Vec<PortStatus>,

    // Interface
    pub interface_name: String,
    pub gateway: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LatencyStatus {
    Good,
    Degraded,
    Unreachable,
    #[allow(dead_code)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DnsStatus {
    Resolved,
    Slow,
    Failed,
    #[allow(dead_code)]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PortStatus {
    pub port: u16,
    pub label: String,
    pub open: bool,
    pub latency_ms: Option<f64>,
}

pub struct AppState {
    pub current: NetworkSnapshot,
    pub history_latency: VecDeque<Option<f64>>,
    pub history_bw_tx: VecDeque<f64>,
    pub history_bw_rx: VecDeque<f64>,
    pub history_dns: VecDeque<Option<f64>>,
    pub max_history: usize,
}

impl AppState {
    pub fn new(snapshot: NetworkSnapshot) -> Self {
        let max_history = 120;
        Self {
            current: snapshot,
            history_latency: VecDeque::with_capacity(max_history),
            history_bw_tx: VecDeque::with_capacity(max_history),
            history_bw_rx: VecDeque::with_capacity(max_history),
            history_dns: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn update(&mut self, snapshot: NetworkSnapshot) {
        if self.history_latency.len() >= self.max_history {
            self.history_latency.pop_front();
        }
        if self.history_bw_tx.len() >= self.max_history {
            self.history_bw_tx.pop_front();
        }
        if self.history_bw_rx.len() >= self.max_history {
            self.history_bw_rx.pop_front();
        }
        if self.history_dns.len() >= self.max_history {
            self.history_dns.pop_front();
        }

        self.history_latency.push_back(snapshot.latency_ms);
        self.history_bw_tx.push_back(snapshot.bandwidth_tx_bps);
        self.history_bw_rx.push_back(snapshot.bandwidth_rx_bps);
        self.history_dns.push_back(snapshot.dns_resolution_ms);

        self.current = snapshot;
    }
}
