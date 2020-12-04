use super::app::{App, InputMode, Status};
use super::util::split_keep;
use chrono::{TimeZone, Local};
use tui::{
    terminal::Frame,
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Wrap},
};


pub fn render<B>(app: &mut App, frame: &mut Frame<B>) where B: Backend{
    // Status bar
    let update_str = match app.status {
        Status::Updating => "Updating...",
        _ => ""
    };
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw(update_str),
            ],
            Style::default(),
        ),
        InputMode::Search => (
            vec![
                Span::raw("/"),
                Span::styled(&app.search_input_raw, Style::default().add_modifier(Modifier::BOLD)),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let status_bar = Paragraph::new(text).style(Style::default().bg(Color::DarkGray));

    // Reader
    let reader = match app.table.state.selected() {
        Some(i) =>  {
            let item = &app.items[i];
            let pub_date = match item.published_at {
                Some(ts) => Local.timestamp(ts, 0).format("%B %d, %Y %H:%M").to_string(),
                None => "<no pub date>".to_string()
            };

            let mut text = vec![
                Spans::from(
                    Span::styled(item.title.as_deref().unwrap_or("<no title>"), Style::default().fg(Color::Yellow))),
                Spans::from(pub_date),
                Spans::from(item.channel.clone()),
                Spans::from("\n"),
            ];

            for line in item.description.as_deref().unwrap_or("<no description>").split('\n') {
                text.push(Spans::from(line));
            }

            Paragraph::new(text.clone())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true })
                .scroll((app.reader_scroll, 0))
        }
        None => Paragraph::new("No item selected.")
    };


    if app.focus_reader {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                 Constraint::Min(1),
                 Constraint::Length(1),
            ].as_ref())
            .split(frame.size());

        frame.render_widget(reader, chunks[0]);
        frame.render_widget(status_bar, chunks[1]);
    } else {
        // Item list
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::White);
        let header_cells = ["Title", "Published"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1);

        let regex = match app.input_mode {
            InputMode::Normal => &app.search_query,
            InputMode::Search => &app.search_input
        };

        let rows = app.table.items.iter().enumerate().map(|(i, item)| {
            let height = item
                .iter()
                .map(|content| content.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(1)
                + 1;
            let cells = item.iter().map(|c| {
                let spans: Vec<Span> = match regex {
                    Some(re) => {
                        let parts = split_keep(re, c);
                        parts.iter().map(|(text, is_match)| {
                            if *is_match {
                                Span::styled(*text, Style::default().fg(Color::Yellow))
                            } else {
                                Span::raw(*text)
                            }
                        }).collect()
                    },
                    None => vec![Span::raw(c)]
                };
                Cell::from(Spans::from(spans))
            });
            let style = if app.items[i].read {
                Style::default().fg(Color::Rgb(100,100,100))
            } else {
                Style::default()
            };
            Row::new(cells).height(height as u16).style(style)
        });
        let item_list = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::BOTTOM))
            .highlight_style(selected_style)
            .widths(&[
                Constraint::Percentage(50),
                Constraint::Length(30),
                Constraint::Max(10),
            ]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                 Constraint::Min(1),
                 Constraint::Percentage(50),
                 Constraint::Length(1),
            ].as_ref())
            .split(frame.size());

        frame.render_stateful_widget(item_list, chunks[0], &mut app.table.state);
        frame.render_widget(reader, chunks[1]);
        frame.render_widget(status_bar, chunks[2]);
    }
}
