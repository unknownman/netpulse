<div align="center">

# ⚡ NetPulse

**Ultra-lightweight, zero-flicker, non-blocking terminal network dashboard in Rust.**

[![Crates.io](https://img.shields.io/crates/v/netpulse.svg?style=flat-square&color=orange)](https://crates.io/crates/netpulse)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/unknownman/netpulse/ci.yml?branch=main&style=flat-square)](https://github.com/unknownman/netpulse/actions)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey.svg?style=flat-square)](https://github.com/unknownman/netpulse)
[![Zero-Privilege](https://img.shields.io/badge/security-zero--privilege-brightgreen.svg?style=flat-square)](https://github.com/unknownman/netpulse)

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#keyboard-shortcuts">Shortcuts</a> •
  <a href="#architecture">Architecture</a> •
  <a href="ROADMAP.md">Roadmap</a>
</p>

</div>

---

## 🚀 Why NetPulse?

Most terminal network tools force you into an unappealing trade-off: heavy background resource consumption, screen flicker during refresh, mandatory `sudo`/root privileges for basic ICMP pings, or complex configuration just to see why your connection feels sluggish.

**NetPulse** is built from the ground up for systems engineers, SREs, and terminal power users:
- **Zero Sudo Required**: Transparent ICMP ping with automated non-blocking TCP fallback (ports 443 & 53) ensures reliable latency diagnostics in locked-down or containerized environments.
- **True Non-Blocking Async**: Powered by Tokio and Hickory DNS; network I/O, DNS benchmarks, port polling, and UI rendering execute independently with zero frame drops.
- **Zero-Flicker Ratatui TUI**: 15 FPS sub-millisecond redraws with zero heap allocations on the render path.
- **Minimal Footprint**: Single static binary (< 4MB stripped), instant startup, and near-zero CPU/memory footprint.

---

## 📺 Preview

```text
┌───────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ NETPULSE  ● 3 ifaces | gw: 192.168.1.1 | dns: 1.1.1.1 | status: active                                    │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ Network Interfaces ──────────────────────────────────────┐┌ Throughput History ───────────────────────────┐
│Interface    RX Rate     TX Rate    Total RX    Total TX  ││ RX: en0 (14.2 MB/s)                           │
│en0          14.2 MB/s   1.8 MB/s   8.42 GB     2.15 GB   ││    █▅▁▂ ▂ ▄▄▁▅                                │
│utun3        820 KB/s    110 KB/s   412.0 MB    98.4 MB   ││▂▅▄█████▃█▆█████                               │
│lo0          0 B/s       0 B/s      185.9 MB    185.9 MB  ││ TX: en0 (1.8 MB/s)                            │
│                                                          ││    ▁          █                               │
└──────────────────────────────────────────────────────────┘└───────────────────────────────────────────────┘
┌ Ping & Latency Probes ──────────────────────────────┐┌ DNS Latency Benchmark (avg: 12.4ms) ───────────────┐
│● 1.1.1.1 11ms [icmp]  ● 8.8.8.8 14ms [icmp]         ││● google.com       8.2ms (142.250.190.46)          │
│● gw:192.168.1.1 1ms [icmp]                          ││● cloudflare.com  11.4ms (104.16.132.229)          │
│min: 1.0ms avg: 8.7ms max: 14.0ms loss: 0%           ││● github.com      17.6ms (140.82.121.4)            │
└─────────────────────────────────────────────────────┘└────────────────────────────────────────────────────┘
┌ Open / Listening Ports (4) ───────────────────────────────────────────────────────────────────────────────┐
│Proto       Port        PID         Process Name                                                           │
│TCP         22          1420        sshd                                                                   │
│TCP         5432        2811        postgres                                                               │
│TCP         8080        4190        node (backend-api)                                                     │
│UDP         5353        312         mDNSResponder                                                          │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────┘
 q / esc quit  ctrl+c exit  ● pulse active
```

---

## ✨ Key Features

| Feature | Description |
|---|---|
| 📊 **Real-Time Bandwidth & Sparklines** | Per-interface RX/TX rates computed accurately over exact time deltas, accompanied by adaptive 30-sample history sparklines. |
| 🌐 **Zero-Privilege Latency Probing** | High-precision ICMP pings with seamless fallback to concurrent non-blocking TCP handshakes (raced across 443 & 53). |
| ⚡ **Parallel DNS Resolution** | Concurrently benchmarks your upstream resolvers against top global domains using `futures::future::join_all`. |
| 🔍 **Open Port & Socket Auditor** | Scans listening TCP/UDP sockets and maps them to active process names without spiking CPU or memory. |
| 🎨 **Adaptive & Responsive UI** | Responsive Ratatui layout that scales cleanly from small terminal splits (40x10) to ultra-wide displays with `NO_COLOR` support. |
| 🛡️ **Endian-Safe Route Detection** | Cross-platform gateway discovery supporting macOS routing tables and Linux `/proc/net/route` on both Little-Endian and Big-Endian architectures. |

---

## 📦 Installation

### Option 1: Via Cargo (from Crates.io)
```bash
cargo install netpulse
```

### Option 2: Via `cargo-binstall` (Fast Pre-built Binary)
```bash
cargo binstall netpulse
```

### Option 3: Download Pre-built Binaries
Download the pre-compiled binary for your platform from the [GitHub Releases](https://github.com/unknownman/netpulse/releases):

```bash
# macOS (Apple Silicon / ARM64)
curl -LO https://github.com/unknownman/netpulse/releases/latest/download/netpulse-aarch64-apple-darwin.tar.gz
tar -xzf netpulse-aarch64-apple-darwin.tar.gz
sudo mv netpulse /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/unknownman/netpulse/releases/latest/download/netpulse-x86_64-unknown-linux-musl.tar.gz
tar -xzf netpulse-x86_64-unknown-linux-musl.tar.gz
sudo mv netpulse /usr/local/bin/
```

### Option 4: Build from Source
```bash
git clone https://github.com/unknownman/netpulse.git
cd netpulse
cargo build --release
sudo cp target/release/netpulse /usr/local/bin/
```

---

## 🐚 Shell Completions

Generate native auto-completion scripts for your shell using `--generate-completions`:

```bash
# Zsh
netpulse --generate-completions zsh > ~/.zfunc/_netpulse
# Add 'fpath+=~/.zfunc' to your ~/.zshrc if not already present

# Bash
netpulse --generate-completions bash > ~/.local/share/bash-completion/completions/netpulse

# Fish
netpulse --generate-completions fish > ~/.config/fish/completions/netpulse.fish
```

---

## 💻 Usage & CLI Flags

```bash
# Launch with default settings (1s refresh, auto-detect active interfaces)
netpulse

# High-frequency refresh (250ms interval)
netpulse -i 250

# Monitor a specific interface only (e.g. en0 or eth0)
netpulse --interface en0

# Show all interfaces including inactive / virtual loopbacks
netpulse --all

# Plain monochrome output (or export NO_COLOR=1)
netpulse --no-color
```

### CLI Reference

| Flag | Short | Default | Description |
|---|---|---|---|
| `--interval <MS>` | `-i` | `1000` | Collector refresh interval in milliseconds |
| `--interface <NAME>` | | `Auto` | Network interface to monitor |
| `--all` | `-a` | `false` | Show all interfaces, including inactive ones |
| `--no-color` | | `false` | Disable ANSI colored output |
| `--generate-completions <SHELL>` | | | Output shell completion script (`bash`, `zsh`, `fish`, `powershell`, `elvish`) |
| `--help` | `-h` | | Print help and options |
| `--version` | `-V` | | Print version |

---

## ⌨️ Keyboard Shortcuts

| Key | Action |
|---|---|
| <kbd>q</kbd> | Quit application and restore terminal |
| <kbd>Esc</kbd> | Quit application and restore terminal |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> | Force exit safely |

---

## 🏗️ Architecture & Zero-Privilege Security

```text
┌────────────────────────────────────────────────────────┐
│                      main.rs                           │
│  (15 FPS Ratatui Loop · Zero Heap Allocation / Render) │
└──────▲─────────────────▲─────────────────▲─────────────┘
       │                 │                 │
┌──────┴──────────┐┌─────┴──────────┐┌─────┴─────────────┐
│ Bandwidth Task  ││  Latency Task  ││    DNS Task       │
│  (sysinfo-rs)   ││ (ICMP + TCP)   ││(Hickory Resolver) │
└─────────────────┘└────────────────┘└───────────────────┘
```

1. **Lock-Free Communication**: Each background collector asynchronously publishes immutable snapshots over `tokio::sync::watch` channels without shared `Mutex` locks or global statics.
2. **True Async Network I/O**: Network probes utilize non-blocking async sockets bounded by deterministic timeouts, preventing unresponsive threads even on broken routing paths.
3. **Safe Sub-millisecond Rendering**: Snapshot states are borrowed directly by reference during UI passes (`watch::Receiver::borrow()`), completely avoiding clone churn and garbage collection pauses.
4. **Zero-Privilege Guarantee**: Uses OS routing sysctls and non-privileged raw ICMP/TCP handshake capabilities provided by modern Linux kernels (`net.ipv4.ping_group_range`) and macOS sockets.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

```bash
# Run test suite
cargo test -- --nocapture

# Run linter
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).
