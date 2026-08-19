use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Row, Sparkline, Table};
use ratatui::Frame;

use crate::app::NetworkSnapshot;

pub fn render(f: &mut Frame, snapshot: &NetworkSnapshot, no_color: bool) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(area);

    draw_header(f, snapshot, chunks[0], no_color);
    draw_table(f, snapshot, chunks[1], no_color);
    draw_footer(f, chunks[3], no_color);
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
            format!("monitoring {} interface{}", count, if count == 1 { "" } else { "s" }),
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
            Span::styled(fmt_bytes_total(iface.total_rx), styled(Color::DarkGray, no_color)),
            Span::styled(fmt_bytes_total(iface.total_tx), styled(Color::DarkGray, no_color)),
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
        Span::styled(" q", styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD)),
        Span::styled(" quit  ", styled(Color::Gray, no_color)),
        Span::styled("ctrl+c", styled(Color::Cyan, no_color).add_modifier(Modifier::BOLD)),
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
