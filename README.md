<div align="center">

# NetPulse

**A fast, lightweight network dashboard for the terminal. Built in Rust.**

[![Crates.io](https://img.shields.io/crates/v/netpulse-tui.svg?style=flat-square&color=orange)](https://crates.io/crates/netpulse-tui)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg?style=flat-square)](LICENSE-MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/unknownman/netpulse/ci.yml?branch=main&style=flat-square)](https://github.com/unknownman/netpulse/actions)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey.svg?style=flat-square)](https://github.com/unknownman/netpulse)

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#keyboard-shortcuts">Shortcuts</a> •
  <a href="#how-it-works">How It Works</a> •
  <a href="ROADMAP.md">Roadmap</a>
</p>

</div>

---

## Why NetPulse?

Most terminal network tools force you to pick your poison: bloated resource usage, flickering screens, the need for root privileges, or sluggish performance. NetPulse doesn't compromise.

It's designed for people who live in the terminal—systems engineers, SREs, and anyone who needs reliable network diagnostics without the hassle:

- **No root required** — Uses non-blocking TCP fallback (ports 443 & 53) alongside ICMP to probe latency, even in locked-down environments
- **Truly async** — Built on Tokio and Hickory DNS. Network I/O, DNS checks, port scanning, and rendering all run independently—no frame drops
- **Zero flicker** — 15 FPS refresh rate with sub-millisecond redraws and zero heap allocations on the render path
- **Lightweight** — A single binary under 4MB. Starts instantly, uses almost no CPU or memory

## Features

- **Real-time bandwidth monitor** — Watch RX/TX rates per interface with scrolling history sparklines
- **Zero-privilege latency probing** — High-precision ICMP pings with automatic fallback to TCP handshakes when needed
- **Parallel DNS resolution** — Benchmark your resolvers against real-world domains concurrently
- **Port & socket auditor** — Scan listening TCP/UDP sockets and see which processes own them
- **Responsive UI** — Works cleanly on tiny terminal splits or wide displays. Supports `NO_COLOR` mode
- **Cross-platform** — Runs on Linux, macOS, and Windows with proper endian handling and platform-specific optimizations

## Preview

<div align="center">
  <img src="assets/demo.gif" alt="NetPulse Demo" width="850">
</div>

## Installation

### From Crates.io (recommended)

```bash
cargo install netpulse-tui
```

### Using cargo-binstall (pre-built binary)

```bash
cargo binstall netpulse-tui
```

### Download a pre-built binary

Grab the latest release for your platform:

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/unknownman/netpulse/releases/latest/download/netpulse-aarch64-apple-darwin.tar.gz
tar -xzf netpulse-aarch64-apple-darwin.tar.gz
sudo mv netpulse /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/unknownman/netpulse/releases/latest/download/netpulse-x86_64-unknown-linux-musl.tar.gz
tar -xzf netpulse-x86_64-unknown-linux-musl.tar.gz
sudo mv netpulse /usr/local/bin/
```

### Build from source

```bash
git clone https://github.com/unknownman/netpulse.git
cd netpulse
cargo build --release
sudo cp target/release/netpulse /usr/local/bin/
```

## Shell Completions

Generate completions for your shell:

```bash
# Zsh
netpulse --generate-completions zsh > ~/.zfunc/_netpulse
# Add 'fpath+=~/.zfunc' to your ~/.zshrc if not already present

# Bash
netpulse --generate-completions bash > ~/.local/share/bash-completion/completions/netpulse

# Fish
netpulse --generate-completions fish > ~/.config/fish/completions/netpulse.fish
```

## Usage

```bash
# Launch with defaults (1 second refresh, auto-detect interfaces)
netpulse

# Refresh every 250ms for more responsive updates
netpulse -i 250

# Monitor a specific interface
netpulse --interface en0

# Show all interfaces, including inactive ones
netpulse --all

# Monochrome output
netpulse --no-color
```

### CLI Options

| Flag | Short | Default | Description |
|---|---|---|---|
| `--interval <MS>` | `-i` | `1000` | Refresh interval in milliseconds |
| `--interface <NAME>` | | `Auto` | Which interface to monitor |
| `--all` | `-a` | `false` | Show all interfaces |
| `--no-color` | | `false` | Disable colored output |
| `--generate-completions <SHELL>` | | | Generate shell completions |
| `--help` | `-h` | | Show help |
| `--version` | `-V` | | Show version |

## Keyboard Shortcuts

| Key | Action |
|---|---|
| <kbd>q</kbd> or <kbd>Esc</kbd> | Quit |
| <kbd>Ctrl</kbd> + <kbd>C</kbd> | Force exit |

## How It Works

NetPulse runs three independent async tasks that communicate without locks:

1. **Bandwidth collector** — Reads interface statistics from the OS
2. **Latency prober** — Sends ICMP or TCP probes and measures response times
3. **DNS resolver** — Benchmarks your configured resolvers in parallel

Each task publishes updates to the UI via lock-free channels. The main render loop borrows these snapshots directly—no cloning, no garbage collection pauses.

The latency probing is particularly clever: it starts with ICMP (when available) but automatically falls back to TCP handshakes on ports 443 and 53 if you don't have the right permissions. This makes it work reliably in containerized environments and restricted networks.

Network I/O uses true non-blocking sockets with deterministic timeouts, so a broken connection won't hang the UI. Everything runs at 15 FPS with virtually no memory overhead.

## Contributing

Contributions welcome! Please submit a pull request.

```bash
# Run tests
cargo test -- --nocapture

# Run the linter
cargo clippy --all-targets --all-features -- -D warnings
```

## License

Dual-licensed under MIT or Apache 2.0 at your option.
