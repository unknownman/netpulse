use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Sparkline, Table};
use ratatui::Frame;

use crate::app::{LatencyMetrics, LatencyStats, NetworkSnapshot, ProbeProtocol};

pub fn render(f: &mut Frame, snapshot: &NetworkSnapshot, latency: &LatencyMetrics, no_color: bool) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(3),    // interfaces table
            Constraint::Length(6), // latency block
            Constraint::Min(1),   // sparklines
            Constraint::Length(2), // footer
        ])
        .split(area);

    draw_header(f, snapshot, chunks[0], no_color);
    draw_table(f, snapshot, chunks[1], no_color);
    draw_latency(f, latency, chunks[2], no_color);
    draw_footer(f, chunks[4], no_color);
}

fn draw_header(f: &mut Frame, snapshot: &NetworkSnapshot, area: Rect, no_color: bool) {
    let title_style = if no_color {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };

    let count = snapshot.interfaces.len();
    let line = Line::from(vec![
        Span::styled(" netpulse ", title_style),
        Span::styled(
            format!(
                "monitoring {} interface{}",
                count,
                if count == 1 { "" } else { "s" }
            ),
            styled(Color::Gray, no_color),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    f.render_widget(ratatui::widgets::Paragraph::new(line).block(block), area);
}

fn draw_table(f: &mut Frame, snapshot: &NetworkSnapshot, area: Rect, no_color: bool) {
    if snapshot.interfaces.is_empty() {
        let block = Block::default()
            .title(" Interfaces ")
            .borders(Borders::ALL)
            .border_style(styled(Color::DarkGray, no_color));
        f.render_widget(
            ratatui::widgets::Paragraph::new("No active interfaces detected."),
            block.inner(area),
        );
        f.render_widget(block, area);
        return;
    }

    let header_cells = ["Interface", "RX/s", "TX/s", "Total RX", "Total TX"]
        .iter()
        .map(|h| {
            Span::styled(
                *h,
                styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let mut rows = Vec::new();
    for iface in &snapshot.interfaces {
        let row = Row::new(vec![
            Span::styled(iface.name.clone(), styled(Color::White, no_color)),
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
        ]);
        rows.push(row);
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Interfaces ")
            .borders(Borders::ALL)
            .border_style(styled(Color::DarkGray, no_color)),
    );

    f.render_widget(table, area);

    let spark_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            snapshot
                .interfaces
                .iter()
                .map(|_| Constraint::Length(2))
                .collect::<Vec<_>>(),
        )
        .split(area);

    for (i, iface) in snapshot.interfaces.iter().enumerate() {
        if i >= spark_chunks.len() {
            break;
        }
        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(spark_chunks[i]);

        draw_sparkline(
            f,
            &iface.rx_history.iter().copied().collect::<Vec<_>>(),
            "RX",
            Color::Green,
            no_color,
            inner[0],
        );
        draw_sparkline(
            f,
            &iface.tx_history.iter().copied().collect::<Vec<_>>(),
            "TX",
            Color::Blue,
            no_color,
            inner[1],
        );
    }
}

fn draw_latency(f: &mut Frame, latency: &LatencyMetrics, area: Rect, no_color: bool) {
    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    // Probe results row
    let probe_line = build_probe_line(latency, no_color);
    f.render_widget(ratatui::widgets::Paragraph::new(probe_line), inner_layout[0]);

    // Stats block
    let stats_block = build_stats_block(&latency.stats, no_color);
    f.render_widget(stats_block, inner_layout[1]);

    // Gateway info
    let gw_line = build_gateway_line(latency, no_color);
    f.render_widget(ratatui::widgets::Paragraph::new(gw_line), inner_layout[2]);
}

fn build_probe_line(latency: &LatencyMetrics, no_color: bool) -> Line<'_> {
    let mut spans = vec![Span::styled(
        "  ",
        Style::default(),
    )];

    for probe in &latency.probes {
        let (icon, color) = if probe.success {
            let c = latency_color(probe.latency_ms, no_color);
            ("●", c)
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
                styled(Color::Gray, no_color),
            ));
        } else {
            spans.push(Span::styled(
                "timeout ",
                styled(Color::Red, no_color),
            ));
        }
        spans.push(Span::styled(
            format!("[{}] ", proto_tag),
            styled(Color::DarkGray, no_color),
        ));
    }

    Line::from(spans)
}

fn build_stats_block(stats: &LatencyStats, no_color: bool) -> Line<'_> {
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
        Span::styled(
            "    min ",
            styled(Color::DarkGray, no_color),
        ),
        Span::styled(
            format!("{:.1}ms", stats.min_ms),
            latency_color(stats.min_ms, no_color),
        ),
        Span::styled(
            "   avg ",
            styled(Color::DarkGray, no_color),
        ),
        Span::styled(
            format!("{:.1}ms", stats.avg_ms),
            avg_color,
        ),
        Span::styled(
            "   max ",
            styled(Color::DarkGray, no_color),
        ),
        Span::styled(
            format!("{:.1}ms", stats.max_ms),
            latency_color(stats.max_ms, no_color),
        ),
        Span::styled(
            "   loss ",
            styled(Color::DarkGray, no_color),
        ),
        Span::styled(
            format!("{:.0}%", stats.loss_pct),
            loss_color,
        ),
    ])
}

fn build_gateway_line(latency: &LatencyMetrics, no_color: bool) -> Line<'_> {
    match &latency.gateway {
        Some(gw) => Line::from(vec![
            Span::styled(
                "  gw ",
                styled(Color::DarkGray, no_color),
            ),
            Span::styled(
                gw.clone(),
                styled(Color::Cyan, no_color),
            ),
            Span::styled(
                "  ",
                Style::default(),
            ),
        ]),
        None => Line::from(vec![Span::styled(
            "  gw detected: none",
            styled(Color::DarkGray, no_color),
        )]),
    }
}

fn draw_sparkline(
    f: &mut Frame,
    data: &[u64],
    label: &str,
    color: Color,
    no_color: bool,
    area: Rect,
) {
    let max = data.iter().copied().max().unwrap_or(1).max(1);
    let spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" {} ", label))
                .borders(Borders::ALL)
                .border_style(styled(Color::DarkGray, no_color)),
        )
        .data(data)
        .max(max)
        .style(Style::default().fg(color));
    f.render_widget(spark, area);
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
        Span::styled(" exit", styled(Color::Gray, no_color)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styled(Color::DarkGray, no_color));

    f.render_widget(ratatui::widgets::Paragraph::new(footer).block(block), area);
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
    if ms < 50.0 {
        Color::Green
    } else if ms <= 150.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn fmt_bytes(bytes_per_sec: f64) -> String {
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

fn fmt_bytes_total(bytes: u64) -> String {
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
