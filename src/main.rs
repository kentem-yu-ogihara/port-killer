mod app;
mod network;
mod ui;

use anyhow::{Context, Result};
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use network::{get_listening_ports, kill_process};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if let Some(port_str) = args.first() {
        let port: u16 = port_str
            .parse()
            .context("Port must be a number between 0 and 65535")?;
        return cli_kill(port);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn cli_kill(port: u16) -> Result<()> {
    let ports = get_listening_ports()?;
    match ports.iter().find(|p| p.port == port) {
        Some(entry) => {
            println!(
                "Killing '{}' (PID {}) on port {}...",
                entry.process_name, entry.pid, port
            );
            if kill_process(entry.pid) {
                println!("Done.");
            } else {
                eprintln!("Failed to kill process.");
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("No process listening on port {port}");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Confirmation dialog active
                if app.confirm.is_some() {
                    match key.code {
                        KeyCode::Char('y') => app.execute_kill()?,
                        KeyCode::Char('n') | KeyCode::Esc => app.cancel_kill(),
                        _ => {}
                    }
                    continue;
                }

                // Search mode
                if app.is_searching {
                    match key.code {
                        KeyCode::Esc => app.exit_search(),
                        KeyCode::Backspace => app.search_backspace(),
                        KeyCode::Up => app.select_prev(),
                        KeyCode::Down => app.select_next(),
                        KeyCode::Char('x') | KeyCode::Delete => app.request_kill(),
                        KeyCode::Char(c) => app.search_push(c),
                        _ => {}
                    }
                    continue;
                }

                // Normal mode
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('/') => app.enter_search(),
                    KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                    KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                    KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Delete => {
                        app.request_kill()
                    }
                    KeyCode::Char('r') => app.refresh()?,
                    _ => {}
                }
            }
        }

        if app.should_auto_refresh() {
            app.refresh()?;
        }
    }
}
