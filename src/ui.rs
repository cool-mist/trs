use std::io::Stdout;

use crate::error::{Result, TrsError};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{prelude::CrosstermBackend, Terminal};

struct AppState {
    exit: bool,
}

pub fn ui(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut app_state = AppState { exit: false };
    loop {
        handle_events(&mut app_state)?;

        if app_state.exit {
            break;
        }

        draw(&app_state, &mut terminal)?;
    }
    Ok(())
}

fn draw(app_state: &AppState, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    todo!()
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
