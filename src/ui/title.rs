use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct TitleWidget;

impl Widget for TitleWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = "Terminal RSS Manager";
        let para = Paragraph::new(title).alignment(Alignment::Center).style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

        para.render(area, buf);
        Block::default().borders(Borders::RIGHT).render(area, buf);
    }
}
