use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::AppState;

pub struct ArticlesWidget<'a> {
    state: &'a AppState,
}

impl<'a> ArticlesWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for ArticlesWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let selected_channel = match self.state.highlighted_channel {
            Some(c) => self.state.channels.get(c),
            None => None,
        };

        let Some(channel) = selected_channel else {
            let para = Paragraph::new("j/k to navigate channels, q to exit")
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Center);
            para.render(area, buf);
            return;
        };

        let count = channel.articles.len();
        let para = Paragraph::new(format!("{} ({} articles)", channel.title, count)).centered();
        para.render(area, buf);

        let height_per_entry = 1;
        let total_articles = area.height / height_per_entry;
        let total_articles = channel.articles.len().min(total_articles as usize);
        let article_rows = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                (0..total_articles)
                    .map(|_| Constraint::Length(height_per_entry))
                    .collect::<Vec<_>>(),
            )
            .split(area)
            .to_vec();

        for ((idx, row), article) in article_rows.into_iter().enumerate().zip(&channel.articles) {
            let current_highlighted = self
                .state
                .highlighted_article
                .filter(|h| *h == idx)
                .is_some();
            let mut lines = Vec::new();
            let id = Span::styled(
                format!("{:>3}. ", idx + 1),
                get_channel_id_style(current_highlighted),
            );
            let title = Span::styled(
                article.title.clone(),
                get_channel_title_style(current_highlighted),
            );

            lines.push(Line::from(vec![id, title]));
            if let Some(pub_date) = article.pub_date {
                let pub_date_text = format!("Published: {}", pub_date);
                let pub_date_span = Span::styled(pub_date_text, get_channel_pub_date_style(false));
                lines.push(Line::from(vec![pub_date_span]));
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
