use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Widget},
};

pub struct TitleWidget;

impl Widget for TitleWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = "Terminal RSS Reader";
        let areas = Layout::default()
            .constraints(Constraint::from_ratios([(1, 3), (1, 3), (1, 3)]))
            .split(area)
            .to_vec();

        let para = Paragraph::new(title).alignment(Alignment::Center).style(
            Style::default()
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

        para.render(areas[1], buf);

        Block::default()
            .style(Style::default().bg(Color::LightCyan))
            .render(area, buf);
    }
}
