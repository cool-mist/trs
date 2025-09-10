use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};
use time::format_description;

use super::AppState;

pub struct ChannelsWidget<'a> {
    state: &'a AppState,
}

pub struct AddChannelWidget<'a> {
    state: &'a str,
}

impl<'a> ChannelsWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for ChannelsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height_per_entry = 1;
        let total_channels = area.height / height_per_entry;
        let total_channels = self.state.channels.len().min(total_channels as usize);
        let channel_rows = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                (0..total_channels)
                    .map(|_| Constraint::Length(height_per_entry))
                    .collect::<Vec<_>>(),
            )
            .split(area)
            .to_vec();

        for ((idx, row), channel) in channel_rows
            .into_iter()
            .enumerate()
            .zip(&self.state.channels)
        {
            let current_highlighted = self
                .state
                .highlighted_channel
                .filter(|h| *h == idx)
                .is_some();
            let mut spans = Vec::new();
            let id = Span::styled(
                format!("{:>3}. ", idx + 1),
                get_channel_id_style(current_highlighted),
            );
            spans.push(id);

            let title = Span::styled(
                channel.title.clone(),
                get_channel_title_style(current_highlighted),
            );
            spans.push(title);

            let format = format_description::parse("[year]-[month]-[day]").unwrap();
            if let Some(article) = channel.articles.first() {
                let pub_date_text = match article.pub_date {
                    Some(date) => format!(" {}", date.format(&format).unwrap()),
                    None => "".to_string(),
                };
                let pub_date = Span::styled(
                    pub_date_text,
                    get_channel_pub_date_style(current_highlighted),
                );
                spans.push(pub_date);
            }

            let para = Paragraph::new(Line::from(spans))
                .block(Block::default())
                .style(get_channel_list_item_block_style(current_highlighted))
                .alignment(Alignment::Left);
            para.render(row, buf);
        }

        if self.state.show_add_channel_ui {
            let add_channel_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints(Constraint::from_percentages(vec![90, 10]))
                .split(area)
                .to_vec();
            AddChannelWidget {
                state: &self.state.add_channel,
            }
            .render(add_channel_area[1], buf);
        }
    }
}

fn get_channel_id_style(highlighted: bool) -> Style {
    if highlighted {
        Style::default()
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn get_channel_list_item_block_style(highlighted: bool) -> Style {
    if highlighted {
        Style::default()
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn get_channel_pub_date_style(highlighted: bool) -> Style {
    if highlighted {
        Style::default()
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

fn get_channel_title_style(highlighted: bool) -> Style {
    if highlighted {
        Style::default()
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}

impl<'a> Widget for AddChannelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title_top(Line::from("Add Channel").centered())
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let para = Paragraph::new(Line::from(self.state)).block(block);

        para.render(area, buf);
    }
}
