use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use sysinfo::Networks;

pub fn detect_active_interface() -> Result<String> {
    let networks = Networks::new_with_refreshed_list();
    networks
        .keys()
        .find(|&name| {
            name != "lo" && name != "lo0" && !name.starts_with("utun")
        })
        .cloned()
        .context("No active network interface found")
}

#[allow(dead_code)]
pub fn get_interface_stats(iface: &str) -> Result<(u64, u64)> {
    let networks = Networks::new_with_refreshed_list();
    let stats = networks
        .get(iface)
        .context(format!("Interface '{}' not found", iface))?;
    Ok((stats.total_transmitted(), stats.total_received()))
}

pub fn get_default_gateway() -> Option<Ipv4Addr> {
    // Try reading the routing table on macOS/Linux
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("route")
            .args(["-n", "get", "default"])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if let Some(ip_str) = line.strip_prefix("gateway: ") {
                return ip_str.parse::<Ipv4Addr>().ok();
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(content) = fs::read_to_string("/proc/net/route") {
            for line in content.lines().skip(1) {
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() > 2 && fields[1] == "00000000" {
                    if let Ok(ip_bytes) = u32::from_str_radix(fields[2], 16) {
                        return Some(Ipv4Addr::from(ip_bytes.to_le_bytes()));
                    }
                }
            }
        }
    }
    None
}
