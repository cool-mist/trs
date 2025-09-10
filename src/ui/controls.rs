use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::{PopupUiAction, UiAction};

pub struct ControlsWidget;

pub fn parse_popup_ui_action(raw_event: Event) -> PopupUiAction {
    match raw_event {
        Event::Key(key_event) => {
            if key_event.kind != KeyEventKind::Press {
                return PopupUiAction::None;
            }

            if key_event.modifiers != KeyModifiers::NONE {
                return PopupUiAction::None;
            }

            return match key_event.code {
                KeyCode::Backspace => PopupUiAction::Backspace,
                KeyCode::Char(c) => PopupUiAction::AddChar(c),
                KeyCode::Enter => PopupUiAction::Submit,
                KeyCode::Esc => PopupUiAction::Close,
                _ => PopupUiAction::None,
            };
        }
        _ => PopupUiAction::None,
    }
}

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
                    KeyCode::Char('n') => UiAction::ToggleDebug,
                    KeyCode::Char('q') => UiAction::Exit,
                    KeyCode::Char('a') => UiAction::ShowAddChannelUi,
                    KeyCode::Char('d') => UiAction::RemoveChannel,
                    KeyCode::Char('r') => UiAction::ToggleReadStatus,
                    KeyCode::Char('s') => UiAction::SyncChannel,
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
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
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
        let controls_text_line_1 = Line::from(vec![
            control!("j/k"),
            description!(" to navigate up/down, "),
            control!("h/l"),
            description!(" to switch between channels and articles, "),
            control!("a"),
            description!(" add a new RSS channel, "),
            control!("s"),
            description!(" sync channel, "),
            control!("d"),
            description!(" delete an RSS channel, "),
            control!("r"),
            description!(" toggle read state of article, "),
        ])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let controls_text_line_2 = Line::from(vec![
            control!("q"),
            description!(" to exit"),
        ])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .centered();
        let para = Paragraph::new(vec![controls_text_line_1, controls_text_line_2])
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);
        para.render(area, buf);
    }
}
