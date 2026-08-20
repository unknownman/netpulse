use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct InterfaceMetrics {
    pub name: String,
    pub rx_bps: f64,
    pub tx_bps: f64,
    pub total_rx: u64,
    pub total_tx: u64,
    pub rx_history: VecDeque<u64>,
    pub tx_history: VecDeque<u64>,
}

#[derive(Debug, Clone)]
pub struct NetworkSnapshot {
    #[allow(dead_code)]
    pub timestamp: Instant,
    pub interfaces: Vec<InterfaceMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeProtocol {
    Icmp,
    Tcp,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub target: String,
    pub protocol: ProbeProtocol,
    pub latency_ms: f64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub min_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub loss_pct: f64,
}

#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub gateway: Option<String>,
    pub probes: Vec<ProbeResult>,
    pub stats: LatencyStats,
}

#[derive(Debug, Clone)]
pub struct ListeningPort {
    pub protocol: String,
    pub port: u16,
    pub pid: Option<u32>,
    pub process_name: String,
    pub established: bool,
}

#[derive(Debug, Clone)]
pub struct PortsMetrics {
    pub listening: Vec<ListeningPort>,
    #[allow(dead_code)]
    pub collected_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct DnsProbeResult {
    pub domain: String,
    pub latency_ms: f64,
    pub success: bool,
    pub resolved_ip: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DnsMetrics {
    pub server: Option<String>,
    pub probes: Vec<DnsProbeResult>,
    pub avg_latency_ms: f64,
    #[allow(dead_code)]
    pub collected_at: std::time::Instant,
}

