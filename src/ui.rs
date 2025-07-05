use std::io::Stdout;

use crate::{
    args::ListChannelArgs,
    commands::{self, TrsEnv},
    error::{Result, TrsError},
    persistence::RssChannelD,
};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

struct ChannelsWidget<'a> {
    channels: &'a Vec<RssChannelD>,
}

impl<'a> Widget for ChannelsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(5),
                Constraint::Length(50),
                Constraint::Fill(1),
                Constraint::Length(5),
            ])
            .split(area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(5),
            ])
            .split(columns[1]);

        let channel_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1); 10])
            .split(rows[1])
            .to_vec();

        Block::default()
            .borders(Borders::RIGHT)
            .render(rows[1], buf);

        for (row, channel) in channel_rows.into_iter().zip(self.channels) {
            let id = Span::styled(
                format!("{}. ", channel.id),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            let title = Span::styled(
                channel.title.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            let line = Line::from(vec![id, title]);
            let para = Paragraph::new(line).block(Block::default());
            para.render(row, buf);
        }
    }
}

struct AppState {
    exit: bool,
    channels: Vec<RssChannelD>,
}

pub fn ui(mut ctx: TrsEnv, mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut app_state = AppState {
        channels: Vec::new(),
        exit: false,
    };

    let channels = commands::list_channels(&mut ctx, &ListChannelArgs { limit: None })?;
    app_state.channels = channels;

    loop {
        draw(&app_state, &mut terminal)?;

        handle_events(&mut app_state)?;

        if app_state.exit {
            break;
        }
    }
    Ok(())
}

fn draw(app_state: &AppState, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal
        .draw(|f| {
            let channel_widget = ChannelsWidget {
                channels: &app_state.channels,
            };

            f.render_widget(channel_widget, f.area());
        })
        .map_err(|e| TrsError::TuiError(e))?;

    Ok(())
}

fn handle_events(state: &mut AppState) -> Result<()> {
    let event = event::read().map_err(|e| TrsError::TuiError(e))?;
    match event {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            state.exit = true;
        }
        _ => {}
    };
    Ok(())
}
