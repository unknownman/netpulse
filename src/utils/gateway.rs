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
    parse_macos_route_output(&stdout)
}

#[allow(dead_code)]
pub fn parse_macos_route_output(stdout: &str) -> Option<Ipv4Addr> {
    for line in stdout.lines() {
        if let Some(ip_str) = line.trim().strip_prefix("gateway: ") {
            return ip_str.trim().parse::<Ipv4Addr>().ok();
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn detect_gateway_linux() -> Option<Ipv4Addr> {
    use std::fs;
    let content = fs::read_to_string("/proc/net/route").ok()?;
    parse_proc_route_content(&content)
}

#[allow(dead_code)]
pub fn parse_proc_route_ip(hex_str: &str) -> Option<Ipv4Addr> {
    let ip_u32 = u32::from_str_radix(hex_str.trim(), 16).ok()?;
    // In Linux /proc/net/route, addresses are printed with %08X from __be32 stored in memory.
    // Converting the parsed host integer back to native-endian bytes (to_ne_bytes())
    // reproduces the exact original octets [b0, b1, b2, b3] on both Little-Endian and Big-Endian architectures.
    Some(Ipv4Addr::from(ip_u32.to_ne_bytes()))
}

#[allow(dead_code)]
pub fn parse_proc_route_content(content: &str) -> Option<Ipv4Addr> {
    for line in content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // Format: Iface Destination Gateway Flags ...
        // Destination == "00000000" indicates the default gateway route
        if fields.len() > 2 && fields[1] == "00000000" {
            if let Some(gw) = parse_proc_route_ip(fields[2]) {
                if !gw.is_unspecified() {
                    return Some(gw);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_route_ip_endianness() {
        // Test 192.168.1.1
        // On Little-Endian, bytes [192, 168, 1, 1] formatted with %08X is "0101A8C0"
        #[cfg(target_endian = "little")]
        {
            let ip = parse_proc_route_ip("0101A8C0").unwrap();
            assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 1));
        }

        // On Big-Endian, bytes [192, 168, 1, 1] formatted with %08X is "C0A80101"
        #[cfg(target_endian = "big")]
        {
            let ip = parse_proc_route_ip("C0A80101").unwrap();
            assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 1));
        }
    }

    #[test]
    fn test_parse_proc_route_content() {
        #[cfg(target_endian = "little")]
        let route_data =
            "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT\n\
                          eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0\n\
                          eth0\t0001A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0\n";

        #[cfg(target_endian = "big")]
        let route_data =
            "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT\n\
                          eth0\t00000000\tC0A80101\t0003\t0\t0\t100\t00000000\t0\t0\t0\n\
                          eth0\tC0A80100\t00000000\t0001\t0\t0\t100\tFFFFFF00\t0\t0\t0\n";

        let gw = parse_proc_route_content(route_data);
        assert_eq!(gw, Some(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn test_parse_macos_route_output() {
        let macos_output = "   route to: default\n\
                            destination: default\n\
                                   mask: default\n\
                                gateway: 192.168.1.254\n\
                                  flags: <UP,GATEWAY,DONE,STATIC,PRCLONING,GLOBAL>\n";
        let gw = parse_macos_route_output(macos_output);
        assert_eq!(gw, Some(Ipv4Addr::new(192, 168, 1, 254)));
    }
}
