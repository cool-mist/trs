use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

use super::AppState;

pub struct ChannelsWidget<'a> {
    state: &'a AppState,
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
            .margin(1)
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
            let mut lines = Vec::new();
            let id = Span::styled(
                format!("{:>3}. ", idx + 1),
                get_channel_id_style(current_highlighted),
            );

            let title = Span::styled(
                channel.title.clone(),
                get_channel_title_style(current_highlighted),
            );

            lines.push(Line::from(vec![id, title]));
            if let Some(article) = channel.articles.first() {
                let pub_date_text = match article.pub_date {
                    Some(date) => format!("Last update: {}", date),
                    None => "".to_string(),
                };
                let pub_date = Span::styled(
                    pub_date_text,
                    get_channel_pub_date_style(current_highlighted),
                );
                lines.push(Line::from(vec![pub_date]));
            }

            let para = Paragraph::new(lines)
                .block(Block::default())
                .style(get_channel_list_item_block_style(current_highlighted))
                .alignment(Alignment::Left);
            para.render(row, buf);
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
            .bg(Color::LightYellow)
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
