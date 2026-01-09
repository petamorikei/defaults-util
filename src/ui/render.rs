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

pub fn render(frame: &mut Frame, app: &App) {
    match &app.screen {
        Screen::Initial => render_initial_screen(frame, app),
        Screen::LoadingFirst | Screen::LoadingSecond => render_loading_screen(frame, app),
        Screen::WaitingForChanges => render_waiting_screen(frame, app),
        Screen::DiffView => render_diff_screen(frame, app),
        Screen::Error(msg) => render_error_screen(frame, msg),
    }
}

fn render_initial_screen(frame: &mut Frame, app: &App) {
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

    // ステータスバー
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

fn render_loading_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // 中央にローディングメッセージを表示
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

fn render_waiting_screen(frame: &mut Frame, app: &App) {
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

fn render_diff_screen(frame: &mut Frame, app: &App) {
    // Changesにフォーカスしていて選択中の変更がある場合、コマンドプレビューを表示
    let show_preview = app.focus == Focus::Diff && app.selected_change().is_some();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(if show_preview {
            vec![
                Constraint::Length(3), // ヘッダー
                Constraint::Min(8),    // メインコンテンツ
                Constraint::Length(5), // コマンドプレビュー
                Constraint::Length(3), // フッター
            ]
        } else {
            vec![
                Constraint::Length(3), // ヘッダー
                Constraint::Min(10),   // メインコンテンツ
                Constraint::Length(3), // フッター
            ]
        })
        .split(frame.area());

    // ヘッダー（ステータスメッセージがあれば表示）
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

    // メインコンテンツ
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    render_domain_list(frame, app, main_chunks[0]);
    render_diff_details(frame, app, main_chunks[1]);

    // コマンドプレビュー（Changesにフォーカス時のみ）
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

    // フッター
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

fn render_domain_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .diff_result
        .as_ref()
        .map(|diff| {
            diff.domain_diffs
                .iter()
                .enumerate()
                .map(|(i, domain_diff)| {
                    let style = if i == app.selected_domain_index && app.focus == Focus::Domain {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else if i == app.selected_domain_index {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    ListItem::new(format!(
                        "{} ({})",
                        domain_diff.domain,
                        domain_diff.changes.len()
                    ))
                    .style(style)
                })
                .collect()
        })
        .unwrap_or_default();

    let border_style = if app.focus == Focus::Domain {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Domains "),
    );
    frame.render_widget(list, area);
}

fn render_diff_details(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .diff_result
        .as_ref()
        .and_then(|diff| diff.domain_diffs.get(app.selected_domain_index))
        .map(|domain_diff| {
            domain_diff
                .changes
                .iter()
                .enumerate()
                .map(|(i, change)| {
                    let (prefix, color) = match change {
                        Change::Added { .. } => ("+", Color::Green),
                        Change::Removed { .. } => ("-", Color::Red),
                        Change::Modified { .. } => ("~", Color::Yellow),
                    };

                    let style = if i == app.selected_diff_index && app.focus == Focus::Diff {
                        Style::default()
                            .fg(Color::Black)
                            .bg(color)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };

                    let text = format_change(change);
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{} ", prefix), style),
                        Span::styled(text, style),
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

    // Changesペインにフォーカス時はタイトルにコピーヒントを表示
    let title = if app.focus == Focus::Diff {
        " Changes (y to copy) "
    } else {
        " Changes "
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title),
    );
    frame.render_widget(list, area);
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
            if s.len() > 30 {
                format!("\"{}...\"", &s[..27])
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
