use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use sysinfo::Networks;
use tokio::sync::watch;

use crate::app::{InterfaceMetrics, NetworkSnapshot};
use crate::cli::Cli;

pub fn calculate_bps(delta_bytes: u64, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        0.0
    } else {
        (delta_bytes as f64) / elapsed_secs
    }
}

pub async fn run_bandwidth_collector(tx: watch::Sender<NetworkSnapshot>, cli: Cli) {
    let mut networks = Networks::new_with_refreshed_list();
    let mut prev_totals: HashMap<String, (u64, u64)> = HashMap::new();
    let mut histories: HashMap<String, (VecDeque<u64>, VecDeque<u64>)> = HashMap::new();
    let mut first_tick = true;
    let mut last_tick = Instant::now();

    loop {
        tokio::time::sleep(Duration::from_millis(cli.interval)).await;
        let now = Instant::now();
        let elapsed_secs = (now - last_tick).as_secs_f64();
        last_tick = now;

        networks.refresh();

        let mut interfaces = Vec::new();

        for (name, data) in networks.iter() {
            let rx = data.total_received();
            let tx_bytes = data.total_transmitted();

            let (delta_rx, delta_tx) = if first_tick {
                (0u64, 0u64)
            } else {
                match prev_totals.get(name) {
                    Some((prev_rx, prev_tx)) => (
                        rx.saturating_sub(*prev_rx),
                        tx_bytes.saturating_sub(*prev_tx),
                    ),
                    None => (0, 0),
                }
            };

            if !cli.all && delta_rx == 0 && delta_tx == 0 {
                prev_totals.insert(name.clone(), (rx, tx_bytes));
                continue;
            }

            let rx_bps = calculate_bps(delta_rx, elapsed_secs);
            let tx_bps = calculate_bps(delta_tx, elapsed_secs);

            prev_totals.insert(name.clone(), (rx, tx_bytes));

            let hist = histories
                .entry(name.clone())
                .or_insert_with(|| (VecDeque::with_capacity(30), VecDeque::with_capacity(30)));

            if !first_tick {
                if hist.0.len() >= 30 {
                    hist.0.pop_front();
                }
                if hist.1.len() >= 30 {
                    hist.1.pop_front();
                }
                hist.0.push_back(rx_bps as u64);
                hist.1.push_back(tx_bps as u64);
            }

            interfaces.push(InterfaceMetrics {
                name: name.clone(),
                rx_bps,
                tx_bps,
                total_rx: rx,
                total_tx: tx_bytes,
                rx_history: hist.0.clone(),
                tx_history: hist.1.clone(),
            });
        }

        first_tick = false;

        let snapshot = NetworkSnapshot {
            timestamp: Instant::now(),
            interfaces,
        };

        if tx.send(snapshot).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bps_standard() {
        assert_eq!(calculate_bps(1000, 1.0), 1000.0);
        assert_eq!(calculate_bps(250, 0.25), 1000.0);
        assert_eq!(calculate_bps(2000, 2.0), 1000.0);
    }

    #[test]
    fn test_calculate_bps_zero_elapsed() {
        assert_eq!(calculate_bps(1000, 0.0), 0.0);
        assert_eq!(calculate_bps(1000, -1.0), 0.0);
    }

    #[test]
    fn test_calculate_bps_zero_delta() {
        assert_eq!(calculate_bps(0, 1.0), 0.0);
    }
}
