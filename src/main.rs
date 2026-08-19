mod app;
mod cli;
mod collectors;
mod ui;

use std::io;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::watch;

use app::NetworkSnapshot;
use cli::Cli;
use collectors::bandwidth::run_bandwidth_collector;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let initial = NetworkSnapshot {
        timestamp: std::time::Instant::now(),
        interfaces: Vec::new(),
    };

    let (tx, rx) = watch::channel(initial);

    tokio::spawn(run_bandwidth_collector(tx, cli.clone()));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = std::time::Duration::from_millis(66);
    let result = run_app(&mut terminal, rx, &cli, tick_rate).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: watch::Receiver<NetworkSnapshot>,
    cli: &Cli,
    tick_rate: std::time::Duration,
) -> anyhow::Result<()> {
    loop {
        {
            let snapshot = rx.borrow().clone();
            terminal.draw(|f| ui::dashboard::render(f, &snapshot, cli.no_color))?;
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
