use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use super::AppState;

pub struct DebugWidget<'a> {
    state: &'a AppState,
}

impl<'a> DebugWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for DebugWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();
        lines.push(format!("last action: {:?}", self.state.last_action));
        lines.push(format!("channels: {}", self.state.channels.len()));
        lines.push(format!("highlighted: {:?}", self.state.highlighted_channel));
        if let Some(h) = self.state.highlighted_channel {
            for channel in &self.state.channels {
                if channel.id as usize == h {
                    lines.push(format!(
                        "highlighted channel: ({},{},{},{:?})",
                        channel.id, channel.title, channel.link, channel.last_update
                    ));

                    if let Some(article) = channel.articles.first() {
                        lines.push(format!(
                            "first article: ({},{},{},{:?})",
                            article.id, article.title, article.link, article.last_update
                        ));
                    }
                }
            }
        }

        if let Some(h) = self.state.highlighted_article {
            lines.push(format!("highlighted article: {}", h));
        }

        let line_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                lines
                    .iter()
                    .map(|_| Constraint::Length(5))
                    .collect::<Vec<Constraint>>(),
            )
            .split(area)
            .to_vec();

        let mut idx = 0;
        for debug_line in lines {
            let para = Paragraph::new(debug_line)
                .wrap(Wrap::default())
                .block(Block::default().borders(Borders::BOTTOM));
            para.render(line_areas[idx], buf);
            idx = idx + 1;
        }
    }
}
