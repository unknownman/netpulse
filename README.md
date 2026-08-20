<div align="center">

# ⚡ NetPulse

**Ultra-lightweight, zero-flicker, non-blocking terminal network dashboard in Rust.**

[![Crates.io](https://img.shields.io/crates/v/netpulse-tui.svg?style=flat-square&color=orange)](https://crates.io/crates/netpulse-tui)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square)](LICENSE-MIT)
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

<div align="center">
  <img src="assets/demo.gif" alt="NetPulse Demo" width="850">
</div>

> **Re-generate the GIF locally:**
> 1. Install [VHS](https://github.com/charmbracelet/vhs) — `brew install vhs`
> 2. Install the Nerd Font — `brew install --cask font-jetbrains-mono-nerd-font`
> 3. Pre-build the release binary — `cargo build --release`
> 4. Record — `vhs assets/demo.tape`

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
Installs the `netpulse` executable binary onto your PATH:
```bash
cargo install netpulse-tui
```

### Option 2: Via `cargo-binstall` (Fast Pre-built Binary)
```bash
cargo binstall netpulse-tui
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

This project is dual-licensed under either:
- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
