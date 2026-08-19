use anyhow::Result;
use sysinfo::Networks;
use std::time::Instant;

pub struct BandwidthCollector {
    networks: Networks,
    last_tx: u64,
    last_rx: u64,
    last_time: Instant,
}

pub struct BandwidthSample {
    pub total_tx: u64,
    pub total_rx: u64,
    pub bps_tx: f64,
    pub bps_rx: f64,
}

impl BandwidthCollector {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        Self {
            networks,
            last_tx: 0,
            last_rx: 0,
            last_time: Instant::now(),
        }
    }

    pub fn sample(&mut self, interface: &str) -> Result<BandwidthSample> {
        self.networks.refresh();
        let stats = self.networks.get(interface);

        let (tx, rx) = match stats {
            Some(s) => (s.total_transmitted(), s.total_received()),
            None => (0, 0),
        };

        let elapsed = self.last_time.elapsed().as_secs_f64();
        let (bps_tx, bps_rx) = if elapsed > 0.0 {
            let dtx = tx.saturating_sub(self.last_tx) as f64;
            let drx = rx.saturating_sub(self.last_rx) as f64;
            (dtx * 8.0 / elapsed, drx * 8.0 / elapsed)
        } else {
            (0.0, 0.0)
        };

        self.last_tx = tx;
        self.last_rx = rx;
        self.last_time = Instant::now();

        Ok(BandwidthSample {
            total_tx: tx,
            total_rx: rx,
            bps_tx,
            bps_rx,
        })
    }
}
