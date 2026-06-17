use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let (chunks, table_idx, footer_idx) = if app.is_searching {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        (c, 2usize, 3usize)
    } else {
        let c = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        (c, 1usize, 2usize)
    };

    render_header(f, chunks[0]);
    if app.is_searching {
        render_search(f, app, chunks[1]);
    }
    render_table(f, app, chunks[table_idx]);
    render_footer(f, app, chunks[footer_idx]);

    if let Some((port, pid, name)) = app.confirm_entry() {
        render_confirm(f, area, port, pid, name);
    }
}

fn render_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new("  Port Killer  —  Listening ports on this machine")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

fn render_search(f: &mut Frame, app: &App, area: Rect) {
    let text = format!(" {}▌", app.search_query);
    let search = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search  [Esc] to clear ")
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
        );
    f.render_widget(search, area);
}

fn render_table(f: &mut Frame, app: &App, area: Rect) {
    let connections = app.filtered_connections();

    let header_cells = ["Port", "PID", "Process", "Proto", "State"]
        .iter()
        .map(|h| {
            Cell::from(*h)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        });
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(conn.port.to_string()),
                Cell::from(conn.pid.to_string()),
                Cell::from(conn.process_name.clone()),
                Cell::from(conn.protocol.clone()),
                Cell::from(conn.state.clone()),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Min(24),
        Constraint::Length(7),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} Listening Port(s) ", connections.len())),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = TableState::default();
    if !connections.is_empty() {
        state.select(Some(app.selected));
    }

    f.render_stateful_widget(table, area, &mut state);

    if connections.is_empty() {
        let msg = if app.search_query.is_empty() {
            "  No listening ports found"
        } else {
            "  No matches"
        };
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 2,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        f.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            inner,
        );
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let status = app.status_message.as_deref().unwrap_or("Ready");

    let hint = if app.is_searching {
        "[↑/↓] Move   [x] Kill   [Esc] Exit search"
    } else {
        "[↑/↓ j/k] Move   [x] Kill   [/] Search   [r] Refresh   [q] Quit"
    };

    let text = format!("  {}  |  {}", status, hint);
    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

fn render_confirm(f: &mut Frame, area: Rect, port: u16, pid: u32, name: &str) {
    let popup = centered_rect(54, 7, area);
    f.render_widget(Clear, popup);

    let text = format!(
        "\n  Kill '{}' (PID {}) listening on port {}?\n\n  [y] Yes — kill it     [n / Esc] Cancel",
        name, pid, port
    );

    let block = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(" Confirm Kill ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        );

    f.render_widget(block, popup);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + r.width.saturating_sub(width) / 2;
    let y = r.y + r.height.saturating_sub(height) / 2;
    Rect {
        x: x.min(r.right()),
        y: y.min(r.bottom()),
        width: width.min(r.width),
        height: height.min(r.height),
    }
}
