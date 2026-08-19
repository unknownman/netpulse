use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::app::{DnsStatus, LatencyStatus};

pub fn sparkline(f: &mut Frame, data: &[u64], color: Color, label: &str, area: Rect) {
    let max = data.iter().copied().max().unwrap_or(1).max(1);
    let spark = ratatui::widgets::Sparkline::default()
        .block(
            Block::default()
                .title(format!(" {} (max:{}) ", label, fmt_val(max)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .data(data)
        .max(max)
        .style(Style::default().fg(color));
    f.render_widget(spark, area);
}

pub fn status_badge(latency: &LatencyStatus) -> Span<'static> {
    match latency {
        LatencyStatus::Good => Span::styled(
            "[ OK ]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        LatencyStatus::Degraded => Span::styled(
            "[WARN]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        LatencyStatus::Unreachable => Span::styled(
            "[FAIL]",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        LatencyStatus::Unknown => Span::styled(
            "[----]",
            Style::default().fg(Color::DarkGray),
        ),
    }
}

pub fn dns_status_badge(status: &DnsStatus) -> Span<'static> {
    match status {
        DnsStatus::Resolved => Span::styled(
            "[ OK ]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        DnsStatus::Slow => Span::styled(
            "[WARN]",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        DnsStatus::Failed => Span::styled(
            "[FAIL]",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        DnsStatus::Unknown => Span::styled(
            "[----]",
            Style::default().fg(Color::DarkGray),
        ),
    }
}

#[allow(dead_code)]
pub fn progress_bar(value: f64, max: f64, width: usize) -> String {
    let ratio = if max > 0.0 { (value / max).min(1.0) } else { 0.0 };
    let filled = (ratio * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn fmt_val(v: u64) -> String {
    if v < 1000 {
        format!("{}", v)
    } else if v < 1_000_000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        format!("{:.1}M", v as f64 / 1_000_000.0)
    }
}
