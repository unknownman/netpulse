use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;

use crate::app::{
    DnsMetrics, LatencyMetrics, LatencyStats, NetworkSnapshot, PortsMetrics, ProbeProtocol,
};

pub fn render(
    f: &mut Frame,
    snapshot: &NetworkSnapshot,
    latency: &LatencyMetrics,
    dns: &DnsMetrics,
    ports: &PortsMetrics,
    no_color: bool,
) {
    let area = f.area();

    // Check minimum terminal dimensions for clean graceful resize handling
    if area.width < 40 || area.height < 10 {
        let msg = vec![
            Line::from(Span::styled(
                "Terminal window too small",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("Current: {}x{} (Min: 40x10)", area.width, area.height),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::raw("Please resize your terminal window.")),
        ];
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(styled(Color::DarkGray, no_color));
        let p = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .block(block);
        f.render_widget(p, area);
        return;
    }

    // Adaptive vertical partitioning
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(7), // Interfaces & Sparklines
            Constraint::Length(7), // Latency & DNS
            Constraint::Min(5),    // Open Ports
            Constraint::Length(2), // Footer
        ])
        .split(area);

    draw_header(f, snapshot, latency, dns, chunks[0], no_color);
    draw_bandwidth_section(f, snapshot, chunks[1], no_color);
    draw_diagnostics_section(f, latency, dns, chunks[2], no_color);
    draw_ports_section(f, ports, chunks[3], no_color);
    draw_footer(f, chunks[4], no_color);
}

fn draw_header(
    f: &mut Frame,
    snapshot: &NetworkSnapshot,
    latency: &LatencyMetrics,
    dns: &DnsMetrics,
    area: Rect,
    no_color: bool,
) {
    let title_style = if no_color {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };

    let count = snapshot.interfaces.len();
    let gw_text = latency.gateway.as_deref().unwrap_or("none");
    let dns_text = dns.server.as_deref().unwrap_or("system");

    let mut line_spans = vec![
        Span::styled(" NETPULSE ", title_style),
        Span::raw(" "),
        Span::styled(
            format!("● {} iface{} ", count, if count == 1 { "" } else { "s" }),
            styled(Color::Green, no_color),
        ),
        Span::styled("| ", styled(Color::DarkGray, no_color)),
        Span::styled("gw: ", styled(Color::DarkGray, no_color)),
        Span::styled(gw_text, styled(Color::Cyan, no_color)),
        Span::raw(" "),
        Span::styled("| ", styled(Color::DarkGray, no_color)),
        Span::styled("dns: ", styled(Color::DarkGray, no_color)),
        Span::styled(dns_text, styled(Color::Blue, no_color)),
    ];

    if area.width > 95 {
        line_spans.push(Span::styled(" | ", styled(Color::DarkGray, no_color)));
        line_spans.push(Span::styled(
            "status: active",
            styled(Color::DarkGray, no_color),
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    f.render_widget(Paragraph::new(Line::from(line_spans)).block(block), area);
}

fn draw_bandwidth_section(f: &mut Frame, snapshot: &NetworkSnapshot, area: Rect, no_color: bool) {
    if area.width >= 100 {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        draw_interfaces_table(f, snapshot, cols[0], no_color);
        draw_sparklines_panel(f, snapshot, cols[1], no_color);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        draw_interfaces_table(f, snapshot, rows[0], no_color);
        draw_sparklines_panel(f, snapshot, rows[1], no_color);
    }
}

fn draw_interfaces_table(f: &mut Frame, snapshot: &NetworkSnapshot, area: Rect, no_color: bool) {
    let block = Block::default()
        .title(" Network Interfaces ")
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    if snapshot.interfaces.is_empty() {
        f.render_widget(
            Paragraph::new("No active network interfaces detected.").block(block),
            area,
        );
        return;
    }

    let header_cells = ["Interface", "RX Rate", "TX Rate", "Total RX", "Total TX"]
        .iter()
        .map(|h| {
            Span::styled(
                *h,
                styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let max_rows = area.height.saturating_sub(2) as usize;
    let rows: Vec<Row> = snapshot
        .interfaces
        .iter()
        .take(max_rows)
        .map(|iface| {
            Row::new(vec![
                Span::styled(iface.name.as_str(), styled(Color::White, no_color)),
                Span::styled(fmt_bytes(iface.rx_bps), styled(Color::Green, no_color)),
                Span::styled(fmt_bytes(iface.tx_bps), styled(Color::Blue, no_color)),
                Span::styled(
                    fmt_bytes_total(iface.total_rx),
                    styled(Color::DarkGray, no_color),
                ),
                Span::styled(
                    fmt_bytes_total(iface.total_tx),
                    styled(Color::DarkGray, no_color),
                ),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(28),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(block);

    f.render_widget(table, area);
}

fn draw_sparklines_panel(f: &mut Frame, snapshot: &NetworkSnapshot, area: Rect, no_color: bool) {
    let block = Block::default()
        .title(" Throughput History ")
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    if snapshot.interfaces.is_empty() {
        f.render_widget(Paragraph::new("No activity").block(block), area);
        return;
    }

    let primary = snapshot
        .interfaces
        .iter()
        .max_by_key(|i| (i.rx_bps + i.tx_bps) as u64)
        .unwrap_or(&snapshot.interfaces[0]);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let mut rx_buf = [0u64; 30];
    let rx_len = primary.rx_history.len().min(30);
    for (i, &val) in primary.rx_history.iter().take(rx_len).enumerate() {
        rx_buf[i] = val;
    }
    let rx_data = &rx_buf[..rx_len];

    let mut tx_buf = [0u64; 30];
    let tx_len = primary.tx_history.len().min(30);
    for (i, &val) in primary.tx_history.iter().take(tx_len).enumerate() {
        tx_buf[i] = val;
    }
    let tx_data = &tx_buf[..tx_len];

    let rx_max = rx_data.iter().copied().max().unwrap_or(1).max(1);
    let tx_max = tx_data.iter().copied().max().unwrap_or(1).max(1);

    let rx_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(
                    " RX: {} ({}) ",
                    primary.name,
                    fmt_bytes(primary.rx_bps)
                ))
                .border_style(styled(Color::DarkGray, no_color)),
        )
        .data(rx_data)
        .max(rx_max)
        .style(Style::default().fg(if no_color {
            Color::Reset
        } else {
            Color::Green
        }));

    let tx_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(
                    " TX: {} ({}) ",
                    primary.name,
                    fmt_bytes(primary.tx_bps)
                ))
                .border_style(styled(Color::DarkGray, no_color)),
        )
        .data(tx_data)
        .max(tx_max)
        .style(Style::default().fg(if no_color {
            Color::Reset
        } else {
            Color::Blue
        }));

    f.render_widget(rx_spark, rows[0]);
    f.render_widget(tx_spark, rows[1]);
}

fn draw_diagnostics_section(
    f: &mut Frame,
    latency: &LatencyMetrics,
    dns: &DnsMetrics,
    area: Rect,
    no_color: bool,
) {
    if area.width >= 90 {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        draw_latency_card(f, latency, cols[0], no_color);
        draw_dns_card(f, dns, cols[1], no_color);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        draw_latency_card(f, latency, rows[0], no_color);
        draw_dns_card(f, dns, rows[1], no_color);
    }
}

fn draw_latency_card(f: &mut Frame, latency: &LatencyMetrics, area: Rect, no_color: bool) {
    let block = Block::default()
        .title(" Ping & Latency Probes ")
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let probe_line = build_probe_line(latency, no_color);
    f.render_widget(Paragraph::new(probe_line), rows[0]);

    let stats_line = build_stats_line(&latency.stats, no_color);
    f.render_widget(Paragraph::new(stats_line), rows[1]);
}

fn build_probe_line(latency: &LatencyMetrics, no_color: bool) -> Line<'_> {
    if latency.probes.is_empty() {
        return Line::from(Span::styled(
            "Probing network targets...",
            styled(Color::DarkGray, no_color),
        ));
    }

    let mut spans = Vec::new();
    for probe in &latency.probes {
        let (icon, color) = if probe.success {
            ("●", latency_color(probe.latency_ms, no_color))
        } else {
            ("○", if no_color { Color::Reset } else { Color::Red })
        };

        let proto_tag = match probe.protocol {
            ProbeProtocol::Icmp => "icmp",
            ProbeProtocol::Tcp => "tcp",
        };

        spans.push(Span::styled(icon, Style::default().fg(color)));
        spans.push(Span::styled(
            format!(" {} ", probe.target),
            styled(Color::White, no_color),
        ));
        if probe.success {
            spans.push(Span::styled(
                format!("{:.0}ms ", probe.latency_ms),
                styled(color, no_color),
            ));
        } else {
            spans.push(Span::styled("loss ", styled(Color::Red, no_color)));
        }
        spans.push(Span::styled(
            format!("[{}]  ", proto_tag),
            styled(Color::DarkGray, no_color),
        ));
    }

    Line::from(spans)
}

fn build_stats_line(stats: &LatencyStats, no_color: bool) -> Line<'_> {
    let loss_color = if no_color {
        Color::Reset
    } else if stats.loss_pct > 30.0 {
        Color::Red
    } else if stats.loss_pct > 10.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let avg_color = latency_color(stats.avg_ms, no_color);

    Line::from(vec![
        Span::styled("min: ", styled(Color::DarkGray, no_color)),
        Span::styled(
            format!("{:.1}ms ", stats.min_ms),
            latency_color(stats.min_ms, no_color),
        ),
        Span::styled("avg: ", styled(Color::DarkGray, no_color)),
        Span::styled(format!("{:.1}ms ", stats.avg_ms), avg_color),
        Span::styled("max: ", styled(Color::DarkGray, no_color)),
        Span::styled(
            format!("{:.1}ms ", stats.max_ms),
            latency_color(stats.max_ms, no_color),
        ),
        Span::styled("loss: ", styled(Color::DarkGray, no_color)),
        Span::styled(format!("{:.0}%", stats.loss_pct), loss_color),
    ])
}

fn draw_dns_card(f: &mut Frame, dns: &DnsMetrics, area: Rect, no_color: bool) {
    let title = format!(
        " DNS Latency Benchmark (avg: {:.1}ms) ",
        dns.avg_latency_ms
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if dns.probes.is_empty() {
        f.render_widget(Paragraph::new("Benchmarking DNS resolution..."), inner);
        return;
    }

    let mut lines = Vec::new();
    let max_lines = inner.height as usize;

    for probe in dns.probes.iter().take(max_lines) {
        let (icon, color) = if probe.success {
            ("●", dns_latency_color(probe.latency_ms, no_color))
        } else {
            ("○", if no_color { Color::Reset } else { Color::Red })
        };

        let mut spans = vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color)),
            Span::styled(
                format!("{:<15}", probe.domain),
                styled(Color::White, no_color),
            ),
        ];

        if probe.success {
            spans.push(Span::styled(
                format!("{:>5.1}ms ", probe.latency_ms),
                styled(color, no_color),
            ));
            if let Some(ref ip) = probe.resolved_ip {
                spans.push(Span::styled(
                    format!("({})", ip),
                    styled(Color::DarkGray, no_color),
                ));
            }
        } else {
            let err_text = probe.error.as_deref().unwrap_or("failed");
            spans.push(Span::styled(
                format!("err: {}", err_text),
                styled(Color::Red, no_color),
            ));
        }

        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_ports_section(f: &mut Frame, ports: &PortsMetrics, area: Rect, no_color: bool) {
    let block = Block::default()
        .title(format!(
            " Open / Listening Ports ({}) ",
            ports.listening.len()
        ))
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    if ports.listening.is_empty() {
        f.render_widget(
            Paragraph::new("Scanning listening ports & sockets...").block(block),
            area,
        );
        return;
    }

    let header_cells = ["Proto", "Port", "PID", "Process Name"]
        .iter()
        .map(|h| {
            Span::styled(
                *h,
                styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let max_rows = area.height.saturating_sub(2) as usize;
    let rows: Vec<Row<'_>> = ports
        .listening
        .iter()
        .take(max_rows)
        .map(|p| {
            let port_color = if p.port < 1024 {
                Color::Magenta
            } else if p.established {
                Color::Green
            } else {
                Color::Yellow
            };
            let pid_str = match p.pid {
                Some(id) => id.to_string(),
                None => "-".into(),
            };
            Row::new(vec![
                Span::styled(p.protocol.as_str(), styled(Color::White, no_color)),
                Span::styled(p.port.to_string(), styled(port_color, no_color)),
                Span::styled(pid_str, styled(Color::DarkGray, no_color)),
                Span::styled(p.process_name.as_str(), styled(Color::Gray, no_color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(block);

    f.render_widget(table, area);
}

fn draw_footer(f: &mut Frame, area: Rect, no_color: bool) {
    let footer = Line::from(vec![
        Span::styled(
            " q",
            styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit  ", styled(Color::Gray, no_color)),
        Span::styled(
            "ctrl+c",
            styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" exit  ", styled(Color::Gray, no_color)),
        Span::styled("●", styled(Color::Green, no_color)),
        Span::styled(" pulse active", styled(Color::DarkGray, no_color)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    f.render_widget(Paragraph::new(footer).block(block), area);
}

fn styled(color: Color, no_color: bool) -> Style {
    if no_color {
        Style::default()
    } else {
        Style::default().fg(color)
    }
}

fn latency_color(ms: f64, no_color: bool) -> Color {
    if no_color {
        return Color::Reset;
    }
    if ms < 40.0 {
        Color::Green
    } else if ms <= 120.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn dns_latency_color(ms: f64, no_color: bool) -> Color {
    if no_color {
        return Color::Reset;
    }
    if ms < 25.0 {
        Color::Green
    } else if ms <= 80.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn fmt_bytes(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{:.0} B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1024.0)
    } else if bytes_per_sec < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn fmt_bytes_total(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_bytes() {
        assert_eq!(fmt_bytes(0.0), "0 B/s");
        assert_eq!(fmt_bytes(512.0), "512 B/s");
        assert_eq!(fmt_bytes(1536.0), "1.5 KB/s");
        assert_eq!(fmt_bytes(1048576.0 * 2.5), "2.5 MB/s");
        assert_eq!(fmt_bytes(1073741824.0 * 1.25), "1.25 GB/s");
    }

    #[test]
    fn test_fmt_bytes_total() {
        assert_eq!(fmt_bytes_total(0), "0 B");
        assert_eq!(fmt_bytes_total(512), "512 B");
        assert_eq!(fmt_bytes_total(2048), "2.0 KB");
        assert_eq!(fmt_bytes_total(1048576 * 3), "3.0 MB");
        assert_eq!(fmt_bytes_total(1073741824 * 5), "5.00 GB");
    }

    #[test]
    fn test_latency_colors() {
        assert_eq!(latency_color(20.0, false), Color::Green);
        assert_eq!(latency_color(40.0, false), Color::Yellow);
        assert_eq!(latency_color(80.0, false), Color::Yellow);
        assert_eq!(latency_color(120.0, false), Color::Yellow);
        assert_eq!(latency_color(120.1, false), Color::Red);
        assert_eq!(latency_color(200.0, false), Color::Red);
        assert_eq!(latency_color(20.0, true), Color::Reset);
    }

    #[test]
    fn test_dns_latency_colors() {
        assert_eq!(dns_latency_color(15.0, false), Color::Green);
        assert_eq!(dns_latency_color(25.0, false), Color::Yellow);
        assert_eq!(dns_latency_color(50.0, false), Color::Yellow);
        assert_eq!(dns_latency_color(80.0, false), Color::Yellow);
        assert_eq!(dns_latency_color(80.1, false), Color::Red);
        assert_eq!(dns_latency_color(120.0, false), Color::Red);
        assert_eq!(dns_latency_color(15.0, true), Color::Reset);
    }
}
