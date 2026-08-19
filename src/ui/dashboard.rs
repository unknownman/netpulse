use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;
use crate::ui::widgets::{dns_status_badge, sparkline, status_badge};

pub fn draw(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(5), // bandwidth
            Constraint::Length(5), // latency
            Constraint::Length(4), // dns
            Constraint::Min(4),   // ports
        ])
        .split(f.area());

    draw_header(f, state, chunks[0]);
    draw_bandwidth(f, state, chunks[1]);
    draw_latency(f, state, chunks[2]);
    draw_dns(f, state, chunks[3]);
    draw_ports(f, state, chunks[4]);
}

fn draw_header(f: &mut Frame, state: &AppState, area: Rect) {
    let snap = &state.current;
    let gw = snap.gateway.as_deref().unwrap_or("N/A");

    let header = Line::from(vec![
        Span::styled(" netpulse ", Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("iface:{}", snap.interface_name), Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled(format!("gw:{}", gw), Style::default().fg(Color::Gray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(header).block(block), area);
}

fn draw_bandwidth(f: &mut Frame, state: &AppState, area: Rect) {
    let snap = &state.current;
    let label = format!(
        "  TX: {}/s   RX: {}/s",
        fmt_bits(snap.bandwidth_tx_bps),
        fmt_bits(snap.bandwidth_rx_bps),
    );

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(10)])
        .split(area);

    let block = Block::default()
        .title(" Bandwidth ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(label).block(block), inner[0]);

    // Sparklines for TX and RX
    let tx_vals: Vec<u64> = state
        .history_bw_tx
        .iter()
        .map(|v| *v as u64)
        .collect();
    let rx_vals: Vec<u64> = state
        .history_bw_rx
        .iter()
        .map(|v| *v as u64)
        .collect();

    sparkline(f, &tx_vals, Color::Green, "TX", inner[1]);
    sparkline(f, &rx_vals, Color::Blue, "RX", inner[1]);
}

fn draw_latency(f: &mut Frame, state: &AppState, area: Rect) {
    let snap = &state.current;
    let latency_str = snap
        .latency_ms
        .map(|v| format!("{:.1}ms", v))
        .unwrap_or_else(|| "---".into());

    let status = status_badge(&snap.latency_status);
    let line = Line::from(vec![
        Span::styled(format!("  ICMP/TCP: {}  ", latency_str), Style::default().fg(Color::White)),
        status,
    ]);

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(10)])
        .split(area);

    let block = Block::default()
        .title(" Latency ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(line).block(block), inner[0]);

    let vals: Vec<u64> = state
        .history_latency
        .iter()
        .map(|v| v.map(|ms| ms as u64).unwrap_or(0))
        .collect();
    sparkline(f, &vals, Color::Yellow, "ms", inner[1]);
}

fn draw_dns(f: &mut Frame, state: &AppState, area: Rect) {
    let snap = &state.current;
    let dns_str = snap
        .dns_resolution_ms
        .map(|v| format!("{:.1}ms", v))
        .unwrap_or_else(|| "---".into());

    let status = dns_status_badge(&snap.dns_status);
    let line = Line::from(vec![
        Span::styled(format!("  Resolver: {}  ", dns_str), Style::default().fg(Color::White)),
        status,
    ]);

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(10)])
        .split(area);

    let block = Block::default()
        .title(" DNS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(line).block(block), inner[0]);

    let vals: Vec<u64> = state
        .history_dns
        .iter()
        .map(|v| v.map(|ms| ms as u64).unwrap_or(0))
        .collect();
    sparkline(f, &vals, Color::Magenta, "ms", inner[1]);
}

fn draw_ports(f: &mut Frame, state: &AppState, area: Rect) {
    let snap = &state.current;
    let mut lines = vec![];

    for p in &snap.ports {
        let icon = if p.open { "●" } else { "○" };
        let color = if p.open { Color::Green } else { Color::Red };
        let lat = p
            .latency_ms
            .map(|v| format!("{:.1}ms", v))
            .unwrap_or_else(|| "-".into());

        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", icon), Style::default().fg(color)),
            Span::styled(format!("{:<5}", p.label), Style::default().fg(Color::White)),
            Span::styled(format!(":{:<5}", p.port), Style::default().fg(Color::DarkGray)),
            Span::styled(lat, Style::default().fg(Color::Gray)),
        ]));
    }

    let block = Block::default()
        .title(" Ports ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn fmt_bits(bps: f64) -> String {
    if bps < 1000.0 {
        format!("{:.0} b", bps)
    } else if bps < 1_000_000.0 {
        format!("{:.1} kbit", bps / 1000.0)
    } else if bps < 1_000_000_000.0 {
        format!("{:.1} Mbit", bps / 1_000_000.0)
    } else {
        format!("{:.1} Gbit", bps / 1_000_000_000.0)
    }
}
