use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, Focus, Screen, StatusKind};
use crate::command::generator::generate_command;
use crate::diff::Change;

pub fn render(frame: &mut Frame, app: &mut App) {
    match app.screen.clone() {
        Screen::Initial => render_initial_screen(frame, app),
        Screen::LoadingFirst | Screen::LoadingSecond => render_loading_screen(frame, app),
        Screen::WaitingForChanges => render_waiting_screen(frame, app),
        Screen::DiffView => render_diff_screen(frame, app),
        Screen::Error(msg) => render_error_screen(frame, &msg),
    }
}

fn render_initial_screen(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("defaults-util - macOS Settings Diff Tool")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from("  1. Press [Enter] to capture the current defaults snapshot"),
        Line::from("  2. Make changes in System Settings"),
        Line::from("  3. Press [Enter] again to capture the second snapshot"),
        Line::from("  4. View the differences and copy commands"),
        Line::from(""),
        Line::from("  Press [q] to quit"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Instructions "),
    );
    frame.render_widget(instructions, chunks[1]);

    // Status bar
    let status_text = if let Some(status) = app.get_status() {
        status.text.as_str()
    } else {
        "Ready - Press [Enter] to start"
    };
    let status_color = if let Some(status) = app.get_status() {
        match status.kind {
            StatusKind::Success => Color::Green,
            StatusKind::Warning => Color::Yellow,
            StatusKind::Info => Color::Cyan,
        }
    } else {
        Color::Green
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color))
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    frame.render_widget(status, chunks[2]);
}

fn render_loading_screen(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Display loading message in center
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(7),
            Constraint::Percentage(35),
        ])
        .split(area);

    let center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

    let msg = app
        .get_status()
        .map(|s| s.text.as_str())
        .unwrap_or("Loading...");

    let loading_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Loading ");

    let loading = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ◐ ", Style::default().fg(Color::Yellow)),
            Span::raw(msg),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Please wait...",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(loading_block);

    frame.render_widget(loading, center[1]);
}

fn render_waiting_screen(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("First Snapshot Captured!")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    let domain_count = app
        .snapshot_before
        .as_ref()
        .map(|s| s.domain_count())
        .unwrap_or(0);

    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::raw(format!("Captured {} domains", domain_count)),
        ]),
        Line::from(""),
        Line::from("  Now make changes in System Settings..."),
        Line::from(""),
        Line::from("  When ready, press [Enter] to capture the second snapshot"),
        Line::from("  and detect changes."),
        Line::from(""),
        Line::from(Span::styled(
            "  [r] Reset  [q] Quit",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Instructions "),
    );
    frame.render_widget(instructions, chunks[1]);

    let status = Paragraph::new("Waiting for changes - Press [Enter] when ready")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    frame.render_widget(status, chunks[2]);
}

fn render_diff_screen(frame: &mut Frame, app: &mut App) {
    // Show command preview when focusing on Changes pane with a selection
    let show_preview = app.focus == Focus::Diff && app.selected_change().is_some();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(if show_preview {
            vec![
                Constraint::Length(3), // Header
                Constraint::Min(8),    // Main content
                Constraint::Length(5), // Command preview
                Constraint::Length(3), // Footer
            ]
        } else {
            vec![
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer
            ]
        })
        .split(frame.area());

    // Header (show status message if available)
    let header_text = if let Some(status) = app.get_status() {
        status.text.clone()
    } else {
        let total_changes = app
            .diff_result
            .as_ref()
            .map(|d| d.total_changes)
            .unwrap_or(0);
        format!("Found {} changes", total_changes)
    };

    let header_color = if let Some(status) = app.get_status() {
        match status.kind {
            StatusKind::Success => Color::Green,
            StatusKind::Warning => Color::Yellow,
            StatusKind::Info => Color::Cyan,
        }
    } else {
        Color::Cyan
    };

    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(header_color)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" Diff View "));
    frame.render_widget(header, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    render_domain_list(frame, app, main_chunks[0]);
    render_diff_details(frame, app, main_chunks[1]);

    // Command preview (only when focused on Changes)
    if show_preview && let Some(change) = app.selected_change() {
        let cmd = generate_command(change);
        let preview = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  $ ", Style::default().fg(Color::DarkGray)),
                Span::styled(cmd, Style::default().fg(Color::White)),
            ]),
        ])
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Command Preview (y to copy) "),
        );
        frame.render_widget(preview, chunks[2]);
    }

    // Footer
    let footer_idx = if show_preview { 3 } else { 2 };
    let footer_text = if app.focus == Focus::Diff {
        "[j/k] Move  [Tab] Switch focus  [y] Copy command  [r] Reset  [q] Quit"
    } else {
        "[j/k] Move  [Tab] Switch focus  [r] Reset  [q] Quit"
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(footer, chunks[footer_idx]);
}

fn render_domain_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .diff_result
        .as_ref()
        .map(|diff| {
            diff.domain_diffs
                .iter()
                .map(|domain_diff| {
                    ListItem::new(format!(
                        "{} ({})",
                        domain_diff.domain,
                        domain_diff.changes.len()
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    let border_style = if app.focus == Focus::Domain {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let highlight_style = if app.focus == Focus::Domain {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Domains "),
        )
        .highlight_style(highlight_style)
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut app.domain_list_state);
}

fn render_diff_details(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .diff_result
        .as_ref()
        .and_then(|diff| diff.domain_diffs.get(app.selected_domain_index))
        .map(|domain_diff| {
            domain_diff
                .changes
                .iter()
                .map(|change| {
                    let (prefix, color) = match change {
                        Change::Added { .. } => ("+", Color::Green),
                        Change::Removed { .. } => ("-", Color::Red),
                        Change::Modified { .. } => ("~", Color::Yellow),
                    };

                    let text = format_change(change);
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{} ", prefix),
                            Style::default().fg(color),
                        ),
                        Span::styled(text, Style::default().fg(color)),
                    ]))
                })
                .collect()
        })
        .unwrap_or_default();

    let border_style = if app.focus == Focus::Diff {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    // Show copy hint in title when focused on Changes pane
    let title = if app.focus == Focus::Diff {
        " Changes (y to copy) "
    } else {
        " Changes "
    };

    let highlight_style = if app.focus == Focus::Diff {
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        )
        .highlight_style(highlight_style)
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut app.diff_list_state);
}

fn format_change(change: &Change) -> String {
    match change {
        Change::Added { key, value, .. } => {
            format!("{}: {}", key, format_value(value))
        }
        Change::Removed { key, old_value, .. } => {
            format!("{}: {}", key, format_value(old_value))
        }
        Change::Modified {
            key,
            old_value,
            new_value,
            ..
        } => {
            format!(
                "{}: {} → {}",
                key,
                format_value(old_value),
                format_value(new_value)
            )
        }
    }
}

fn format_value(value: &plist::Value) -> String {
    match value {
        plist::Value::Boolean(b) => format!("{}", b),
        plist::Value::Integer(i) => format!("{}", i.as_signed().unwrap_or(0)),
        plist::Value::Real(f) => format!("{:.2}", f),
        plist::Value::String(s) => {
            if s.chars().count() > 30 {
                format!("\"{}...\"", s.chars().take(27).collect::<String>())
            } else {
                format!("\"{}\"", s)
            }
        }
        plist::Value::Data(d) => format!("<data {} bytes>", d.len()),
        plist::Value::Array(a) => format!("[{} items]", a.len()),
        plist::Value::Dictionary(d) => format!("{{{}}} keys", d.len()),
        plist::Value::Date(d) => d.to_xml_format().to_string(),
        plist::Value::Uid(u) => format!("UID({})", u.get()),
        _ => "<unknown>".to_string(),
    }
}

fn render_error_screen(frame: &mut Frame, msg: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    let error = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ✗ ", Style::default().fg(Color::Red)),
            Span::raw(msg),
        ]),
    ])
    .style(Style::default().fg(Color::Red))
    .wrap(Wrap { trim: false })
    .block(Block::default().borders(Borders::ALL).title(" Error "));
    frame.render_widget(error, chunks[0]);

    let help = Paragraph::new("Press [r] to reset or [q] to quit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(help, chunks[1]);
}
