# 🗺️ NetPulse Development Roadmap

This document outlines the planned milestones, upcoming features, and architectural goals for **NetPulse**.

---

## 🎯 Release Schedule Overview

| Milestone | Target Version | Focus Area | Status |
|---|---|---|---|
| **Phase 1** | `v0.1.0` | Production MVP & Zero-Churn Async Foundation | ✅ **Completed** |
| **Phase 2** | `v0.2.0` | IPv6, Custom Targets, Process Management & Config | 🚧 **In Progress** |
| **Phase 3** | `v0.3.0` | Headless Streaming, Daemon Mode & Prometheus Exporter | 📋 **Planned** |
| **Phase 4** | `v1.0.0` | eBPF Packet Inspection & Extensible Plugin Engine | 🔭 **Future** |

---

## Phase 1: MVP Foundation (`v0.1.0`) — ✅ Completed

- [x] **Zero-Allocation Render Engine**: Lock-free 15 FPS terminal UI loop powered by `ratatui` and `crossterm`.
- [x] **Non-Blocking Concurrent Latency Engine**:
  - Unprivileged ICMP pinging with `surge-ping`.
  - Non-blocking TCP fallback (concurrent racing across ports 443 & 53).
  - Rolling-window packet loss and latency statistics.
- [x] **High-Precision Bandwidth Tracking**: Exact delta throughput calculation independent of tick interval.
- [x] **Parallel DNS Benchmarking**: Concurrent resolution of benchmark domains via `hickory-resolver` and `futures::future::join_all`.
- [x] **Zero-Privilege Socket & Port Auditor**: Listening socket mapping with persistent `sysinfo` state.
- [x] **Endian-Safe Route Detection**: Linux `/proc/net/route` and macOS route table support.
- [x] **Shell Completions**: Automatic completion generation for Bash, Zsh, Fish, PowerShell, and Elvish.

---

## Phase 2: Configuration & Diagnostics (`v0.2.0`) — 🚧 In Progress

- [ ] **Full IPv6 Support**:
  - Dual-stack ICMPv6 probing.
  - Native IPv6 route and gateway detection.
  - IPv6 DNS benchmarking targets (`2606:4700:4700::1111`, `2001:4860:4860::8888`).
- [ ] **Custom Probe Configuration (`~/.config/netpulse/config.yaml`)**:
  - User-defined ping targets (custom IPs, domain hostnames, internal subnets).
  - Custom DNS benchmark list.
  - Configurable rolling history window and sampling rates.
- [ ] **Interactive Port Management**:
  - Search/filter listening ports with `/` in the TUI.
  - View detailed process metadata (`PID`, user, memory usage, command line).
  - Send termination signals (`SIGTERM` / `SIGKILL`) directly to listening processes from the UI (with confirmation modal).
- [ ] **Per-Process Network Usage (macOS & Linux)**:
  - Identify top bandwidth-consuming processes in real time.

---

## Phase 3: Headless & Observability (`v0.3.0`) — 📋 Planned

- [ ] **Headless JSON & NDJSON Output (`--format json`)**:
  - Stream continuous network telemetry in newline-delimited JSON for piping into `jq`, log aggregators, or external dashboards.
- [ ] **Background Daemon Mode (`--daemon`)**:
  - Run NetPulse in the background as a lightweight systemd / launchd service.
- [ ] **Built-in Prometheus Exporter (`--prometheus-port <PORT>`)**:
  - Expose `/metrics` endpoint with Prometheus counters and gauges:
    - `netpulse_rx_bytes_total` / `netpulse_tx_bytes_total`
    - `netpulse_latency_ms{target="..."}`
    - `netpulse_packet_loss_ratio{target="..."}`
    - `netpulse_dns_latency_ms{domain="..."}`
- [ ] **Historical Export & Snapshots**:
  - Export session network diagnostics as `.csv` or `.json` report upon exit (`--export-on-exit`).

---

## Phase 4: Kernel-Level Observability (`v1.0.0`) — 🔭 Future

- [ ] **Optional eBPF Kernel Probe Module (Linux)**:
  - Fine-grained per-TCP connection latency and retransmission tracking (`tcp_retransmit_skb`).
  - Zero-overhead kernel-level socket tracking using `Aya` / `libbpf-rs`.
- [ ] **Extensible Plugin & Alerting System**:
  - Trigger webhook / desktop notifications when packet loss exceeds threshold or gateway becomes unreachable.
  - Custom WASM/Lua diagnostic collector plugins.
- [ ] **Cross-Platform Static Packaging**:
  - Official Homebrew formula, Arch AUR package, Debian/Ubuntu `.deb`, Alpine `.apk`, and Windows `winget` distribution.

---

## 💬 Feature Requests & Discussion

Have an idea or want to prioritize a feature? Open an issue or start a discussion on our [GitHub Discussions](https://github.com/unknownman/netpulse/discussions) page!
