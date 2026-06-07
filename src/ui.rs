use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::App;
use crate::config;
use crate::search;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    draw_title_bar(frame, chunks[0]);
    draw_input(frame, chunks[1], app);
    draw_results(frame, chunks[2], app);
    draw_status_bar(frame, chunks[3], app);

    if let Some((ref msg, _)) = app.notification {
        draw_notification(frame, chunks[2], msg);
    }
}

fn draw_title_bar(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(Span::styled(
        " Domain Search Aggregator ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(ratatui::style::Modifier::BOLD),
    )))
    .style(Style::default().bg(Color::DarkGray).fg(Color::Cyan))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &App) {
    let input_style = if app.loading {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    let prefix = "> ";
    let display = if app.input.is_empty() && !app.loading {
        format!("{}coffeeshop", prefix)
    } else {
        format!("{}{}", prefix, app.input)
    };

    let input_block = Block::default().borders(Borders::ALL);
    let inner = input_block.inner(area);
    let input = Paragraph::new(display.as_str())
        .style(input_style)
        .block(input_block);
    frame.render_widget(input, area);

    if !app.loading && app.notification.is_none() {
        let cursor_x = inner.x + app.input.len() as u16 + prefix.len() as u16;
        let cursor_y = inner.y;
        frame.set_cursor_position(ratatui::prelude::Position::new(
            cursor_x.min(area.right().saturating_sub(1)),
            cursor_y,
        ));
    }
}

fn draw_results(frame: &mut Frame, area: Rect, app: &App) {
    if app.input.is_empty() && app.results.is_empty() && !app.has_searched {
        let help = Paragraph::new(
            " Type a domain name (e.g. \"coffeeshop\") and press Enter to search\n\
             \n\
             Tab: sort by domain/price/status  ·  ↑↓: navigate  ·  Esc: clear\n\
             Ctrl+C: quit\n\
             \n\
             Pricing data: TLDwise.com (updated daily)",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" Help "));
        frame.render_widget(help, area);
        return;
    }

    if app.loading {
        let spinner = Paragraph::new(" Searching...")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(spinner, area);
        return;
    }

    if app.results.is_empty() && app.has_searched {
        let no_results = Paragraph::new(" No results found. Try a different name.")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(no_results, area);
        return;
    }

    if let Some(ref err) = app.error {
        let err_widget = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title(" Error "));
        frame.render_widget(err_widget, area);
        return;
    }

    let visible_regs = visible_registrar_count(area.width);
    let selected_regs = &config::REGISTRARS[..visible_regs];

    let mut widths = vec![
        Constraint::Min(14),
        Constraint::Length(8),
        Constraint::Min(9),
    ];
    for _ in selected_regs {
        widths.push(Constraint::Length(9));
    }

    let col_selected = app.selected_column;

    let header_labels: Vec<&str> = std::iter::once("Domain")
        .chain(std::iter::once("Status"))
        .chain(std::iter::once("Best"))
        .chain(selected_regs.iter().map(|r| r.label))
        .collect();

    let header_cells: Vec<Cell> = header_labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            let is_col = i >= 3 && col_selected == Some(i - 3);
            let style = if is_col {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)
            };
            Cell::from(Line::from(Span::styled(label, style)))
        })
        .collect();

    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .results
        .iter()
        .map(|r| {
            let status = search::status_symbol(r.available);
            let status_style = match r.available {
                Some(true) => Style::default().fg(Color::Green),
                Some(false) => Style::default().fg(Color::Red),
                None => Style::default().fg(Color::Yellow),
            };

            let best_text = match r.best.as_ref() {
                Some(b) => match b.price {
                    Some(p) => format!("${:.2} {}", p, b.name),
                    None => "—".to_string(),
                },
                None => "—".to_string(),
            };
            let best_style = if best_text == "—" {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD)
            };

            let mut cells = vec![
                Cell::from(Line::from(Span::styled(
                    &r.domain,
                    Style::default().fg(Color::White),
                ))),
                Cell::from(Line::from(Span::styled(status, status_style))),
                Cell::from(Line::from(Span::styled(best_text, best_style))),
            ];

            for reg in selected_regs {
                let price_text = match r.price_map.get(reg.slug) {
                    Some(p) => {
                        let reg_price = p.register.or(p.renew);
                        crate::pricing::format_price(reg_price)
                    }
                    None => "—".to_string(),
                };
                let price_style = if price_text == "—" {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Green)
                };
                cells.push(Cell::from(Line::from(Span::styled(
                    price_text,
                    price_style,
                ))));
            }

            Row::new(cells).height(1)
        })
        .collect();

    let table_title = format!(
        " Results ({}/{}) — sorted by {} ",
        app.stats.available,
        app.stats.total,
        app.sort_by.label()
    );

    let table = Table::new(rows, &widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(table_title))
        .row_highlight_style(Style::default().fg(Color::Black).bg(Color::LightBlue))
        .highlight_symbol("▸ ");

    let mut state = TableState::default().with_selected(Some(app.selected_row));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let text = app.status_text();
    let status = Paragraph::new(Line::from(Span::styled(
        text,
        Style::default().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(status, chunks[0]);

    let hints = Paragraph::new(Line::from(Span::styled(
        " q:quit  Enter:search/open  Tab:sort  ↑↓:nav  ←→:col  +:cheapest  Esc:clear",
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(Alignment::Right)
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(hints, chunks[1]);
}

fn draw_notification(frame: &mut Frame, area: Rect, msg: &str) {
    let width = msg.len() as u16 + 4;
    let height = 3;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + area.height.saturating_sub(height + 2);

    let popup_area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, popup_area);
    let popup = Paragraph::new(Line::from(Span::styled(
        msg,
        Style::default()
            .fg(Color::Black)
            .add_modifier(ratatui::style::Modifier::BOLD),
    )))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .style(Style::default().bg(Color::LightYellow)),
    );
    frame.render_widget(popup, popup_area);
}

fn visible_registrar_count(term_width: u16) -> usize {
    let min_domain_col = 14;
    let status_col = 8;
    let best_col = 9;
    let padding = 6;
    let available = term_width as i32 - min_domain_col - status_col - best_col - padding;
    if available <= 0 {
        return 1;
    }
    let per_reg = 9;
    let count = (available / per_reg) as usize;
    count.min(config::REGISTRARS.len()).max(2)
}
