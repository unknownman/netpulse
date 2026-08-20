use std::time::Duration;

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use sysinfo::System;
use tokio::sync::watch;

use crate::app::{ListeningPort, PortsMetrics};

fn collect_listening_ports(sys: &mut System) -> Vec<ListeningPort> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

    let sockets = match get_sockets_info(af_flags, proto_flags) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut ports = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for entry in &sockets {
        let (protocol, local_port, is_listening) = match &entry.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => {
                let listening = tcp.state == TcpState::Listen;
                ("TCP", tcp.local_port, listening)
            }
            ProtocolSocketInfo::Udp(udp) => ("UDP", udp.local_port, true),
        };

        if !is_listening {
            continue;
        }

        if !seen.insert((protocol, local_port)) {
            continue;
        }

        let pid = entry.associated_pids.first().copied();
        let process_name = match pid {
            Some(p) => sys
                .process(sysinfo::Pid::from_u32(p))
                .map(|proc_| proc_.name().to_string_lossy().into_owned())
                .unwrap_or_else(|| format!("pid:{}", p)),
            None => "[Protected]".into(),
        };

        ports.push(ListeningPort {
            protocol: protocol.into(),
            port: local_port,
            pid,
            process_name,
            established: false,
        });
    }

    ports.sort_by_key(|a| a.port);
    ports
}

pub async fn run_ports_collector(tx: watch::Sender<PortsMetrics>) {
    let mut sys = System::new();
    loop {
        let (ports, returned_sys) = tokio::task::spawn_blocking(move || {
            let ports = collect_listening_ports(&mut sys);
            (ports, sys)
        })
        .await
        .unwrap_or_else(|_| (Vec::new(), System::new()));

        sys = returned_sys;

        let metrics = PortsMetrics {
            listening: ports,
            collected_at: std::time::Instant::now(),
        };

        if tx.send(metrics).is_err() {
            break;
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
