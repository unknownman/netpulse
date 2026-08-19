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
