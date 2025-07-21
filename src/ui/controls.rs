use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::UiAction;

pub struct ControlsWidget;

pub fn parse_ui_action(raw_event: Event) -> UiAction {
    match raw_event {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return UiAction::None;
            }

            if key_event.modifiers == KeyModifiers::CONTROL {
                return match key_event.code {
                    KeyCode::Char('p') => UiAction::FocusEntryUp,
                    KeyCode::Char('n') => UiAction::FocusEntryDown,
                    KeyCode::Char('l') => UiAction::FocusPaneRight,
                    KeyCode::Char('h') => UiAction::FocusPaneLeft,
                    _ => UiAction::None,
                };
            }

            if key_event.modifiers == KeyModifiers::NONE {
                return match key_event.code {
                    KeyCode::Char('l') => UiAction::FocusPaneRight,
                    KeyCode::Char('h') => UiAction::FocusPaneLeft,
                    KeyCode::Char('k') => UiAction::FocusEntryUp,
                    KeyCode::Char('j') => UiAction::FocusEntryDown,
                    KeyCode::Char('d') => UiAction::ToggleDebug,
                    KeyCode::Char('q') => UiAction::Exit,
                    KeyCode::Enter => UiAction::OpenArticle,
                    _ => UiAction::None,
                };
            }

            UiAction::None
        }
        _ => UiAction::None,
    }
}

macro_rules! control {
    ($key:literal) => {
        Span::styled(
            $key,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    };
}

macro_rules! description {
    ($key:literal) => {
        Span::raw($key)
    };
}

impl Widget for ControlsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let controls_text = Line::from(vec![
            control!("j/k"),
            description!(" to navigate up/down, "),
            control!("h/l"),
            description!(" to switch between channels and articles, "),
            control!("q"),
            description!(" to exit"),
        ])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
        let para = Paragraph::new(controls_text)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center);
        para.render(area, buf);
    }
}
