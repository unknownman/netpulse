use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use sysinfo::Networks;
use tokio::sync::watch;

use crate::app::{InterfaceMetrics, NetworkSnapshot};
use crate::cli::Cli;

pub async fn run_bandwidth_collector(tx: watch::Sender<NetworkSnapshot>, cli: Cli) {
    let mut networks = Networks::new_with_refreshed_list();
    let mut prev_totals: HashMap<String, (u64, u64)> = HashMap::new();
    let mut histories: HashMap<String, (VecDeque<u64>, VecDeque<u64>)> = HashMap::new();
    let mut first_tick = true;

    loop {
        tokio::time::sleep(Duration::from_millis(cli.interval)).await;
        networks.refresh();

        let mut interfaces = Vec::new();

        for (name, data) in networks.iter() {
            let rx = data.total_received();
            let tx = data.total_transmitted();

            let (delta_rx, delta_tx) = if first_tick {
                (0u64, 0u64)
            } else {
                match prev_totals.get(name) {
                    Some((prev_rx, prev_tx)) => (
                        rx.saturating_sub(*prev_rx),
                        tx.saturating_sub(*prev_tx),
                    ),
                    None => (0, 0),
                }
            };

            if !cli.all && delta_rx == 0 && delta_tx == 0 {
                prev_totals.insert(name.clone(), (rx, tx));
                continue;
            }

            let rx_bps = delta_rx as f64;
            let tx_bps = delta_tx as f64;

            prev_totals.insert(name.clone(), (rx, tx));

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
                total_tx: tx,
                rx_history: hist.0.clone(),
                tx_history: hist.1.clone(),
            });
        }

        first_tick = false;

        let snapshot = NetworkSnapshot {
            timestamp: std::time::Instant::now(),
            interfaces,
        };

        if tx.send(snapshot).is_err() {
            break;
        }
    }
}
