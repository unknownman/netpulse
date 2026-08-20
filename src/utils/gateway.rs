use std::net::Ipv4Addr;

pub fn detect_gateway() -> Option<Ipv4Addr> {
    #[cfg(target_os = "macos")]
    {
        detect_gateway_macos()
    }
    #[cfg(target_os = "linux")]
    {
        detect_gateway_linux()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn detect_gateway_macos() -> Option<Ipv4Addr> {
    use std::process::Command;
    let output = Command::new("route")
        .args(["-n", "get", "default"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(ip_str) = line.trim().strip_prefix("gateway: ") {
            return ip_str.parse::<Ipv4Addr>().ok();
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn detect_gateway_linux() -> Option<Ipv4Addr> {
    use std::fs;
    let content = fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() > 2 && fields[1] == "00000000" {
            if let Ok(ip_bytes) = u32::from_str_radix(fields[2], 16) {
                return Some(Ipv4Addr::from(ip_bytes.to_le_bytes()));
            }
        }
    }
    None
}
