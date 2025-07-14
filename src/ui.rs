use std::io::Stdout;

use crate::{
    args::{ListChannelArgs, UiArgs},
    commands::{self, TrsEnv},
    error::{Result, TrsError},
    persistence::RssChannelD,
};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

struct AppState {
    exit: bool,
    debug_enabled: bool,
    debug: bool,
    channels: Vec<RssChannelD>,
    focussed: FocussedPane,
    highlighted_channel: Option<usize>,
    highlighted_article: Option<usize>,
}

#[derive(Clone, Copy, PartialEq)]
enum FocussedPane {
    Channels,
    Articles,
}

impl AppState {
    fn is_debug_mode(&self) -> bool {
        self.debug_enabled && self.debug
    }
}

enum TrsEvent {
    None,
    FocusEntryDown,
    FocusEntryUp,
    FocusArticles,
    FocusChannels,
    ToggleDebug,
    Exit,
}

pub fn ui(ctx: &mut TrsEnv, args: &UiArgs) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app_state = AppState {
        channels: Vec::new(),
        exit: false,
        debug_enabled: args.debug,
        debug: false,
        focussed: FocussedPane::Channels,
        highlighted_article: None,
        highlighted_channel: None,
    };

    let channels = commands::list_channels(ctx, &ListChannelArgs { limit: None })?;
    app_state.channels = channels;

    loop {
        draw(&app_state, &mut terminal)?;

        handle_events(&mut app_state)?;

        if app_state.exit {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(app_state: &AppState, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal
        .draw(|f| {
            let root_widget = RootWidget { state: &app_state };
            f.render_widget(root_widget, f.area());
        })
        .map_err(|e| TrsError::TuiError(e))?;

    Ok(())
}

fn handle_events(state: &mut AppState) -> Result<()> {
    let raw_event = event::read().map_err(|e| TrsError::TuiError(e))?;
    let event = parse_trs_event(state, raw_event);
    match event {
        TrsEvent::None => {}
        TrsEvent::FocusEntryDown => match state.highlighted_channel {
            Some(h) => match state.focussed {
                FocussedPane::Channels => {
                    let max = state.channels.len() - 1;
                    state.highlighted_channel = Some(saturating_add(h, 1, max));
                    match state.highlighted_article {
                        Some(a) => {
                            if let Some(c_a) = state.highlighted_channel {
                                if let Some(channel) = state.channels.get(c_a) {
                                    let max_article = channel.articles.len() - 1;
                                    state.highlighted_article =
                                        Some(saturating_add(a, 0, max_article));
                                }
                            }
                        }
                        None => {}
                    }
                }
                FocussedPane::Articles => {
                    if let Some(channel) = state.channels.get(h) {
                        match state.highlighted_article {
                            Some(a) => {
                                let max = channel.articles.len() - 1;
                                state.highlighted_article = Some(saturating_add(a, 1, max));
                            }
                            None => {
                                if !channel.articles.is_empty() {
                                    state.highlighted_article = Some(0);
                                }
                            }
                        }
                    }
                }
            },
            None => {
                if let FocussedPane::Channels = state.focussed {
                    if !state.channels.is_empty() {
                        state.highlighted_channel = Some(0);
                    }

                    match state.highlighted_article {
                        Some(a) => {
                            if let Some(c_a) = state.highlighted_channel {
                                if let Some(channel) = state.channels.get(c_a) {
                                    let max_article = channel.articles.len() - 1;
                                    state.highlighted_article =
                                        Some(saturating_add(a, 0, max_article));
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        },
        TrsEvent::FocusEntryUp => match &mut state.highlighted_channel {
            Some(h) => match state.focussed {
                FocussedPane::Channels => {
                    state.highlighted_channel = Some(saturating_sub(*h, 1, 0));
                }
                FocussedPane::Articles => {
                    if let Some(idx) = state.highlighted_article {
                        state.highlighted_article = Some(saturating_sub(idx, 1, 0));
                    }
                }
            },
            None => {}
        },
        TrsEvent::ToggleDebug => state.debug = !state.debug,
        TrsEvent::Exit => state.exit = true,
        TrsEvent::FocusArticles => state.focussed = FocussedPane::Articles,
        TrsEvent::FocusChannels => state.focussed = FocussedPane::Channels,
    };
    Ok(())
}

fn parse_trs_event(state: &AppState, raw_event: Event) -> TrsEvent {
    match raw_event {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => TrsEvent::Exit,
            KeyCode::Char('j') => TrsEvent::FocusEntryDown,
            KeyCode::Char('l') => match state.focussed {
                FocussedPane::Channels => TrsEvent::FocusArticles,
                FocussedPane::Articles => TrsEvent::None,
            },
            KeyCode::Char('h') => match state.focussed {
                FocussedPane::Channels => TrsEvent::None,
                FocussedPane::Articles => TrsEvent::FocusChannels,
            },
            KeyCode::Char('k') => TrsEvent::FocusEntryUp,
            KeyCode::Char('d') if state.debug_enabled => return TrsEvent::ToggleDebug,
            _ => TrsEvent::None,
        },
        _ => TrsEvent::None,
    }
}

fn saturating_sub(num: usize, to_sub: usize, min: usize) -> usize {
    if num < to_sub {
        min
    } else {
        num - to_sub
    }
}

fn saturating_add(num: usize, to_add: usize, max: usize) -> usize {
    if num + to_add > max {
        max
    } else {
        num + to_add
    }
}

/// ------------------------------------------------------------------------
///                             Widgets
/// ------------------------------------------------------------------------

struct RootWidget<'a> {
    state: &'a AppState,
}
impl<'a> RootWidget<'a> {
    fn get_child_widget_style(&self, arg: &'a str, focussed: bool) -> Block<'a> {
        let title = Line::from(arg)
            .centered()
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        if focussed {
            return Block::default()
                .title_top(title)
                .border_style(Style::default().fg(Color::Blue))
                .borders(Borders::ALL);
        }

        Block::default()
            .title_top(title)
            .border_style(Style::default().fg(Color::DarkGray))
    }
}

struct ChannelsWidget<'a> {
    state: &'a AppState,
}

struct ArticlesWidget<'a> {
    state: &'a AppState,
}

struct ControlsWidget;

struct TitleWidget;

struct DebugWidget<'a> {
    state: &'a AppState,
}

impl<'a> Widget for RootWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut horizontal_constraints = vec![Constraint::Percentage(100)];
        if self.state.is_debug_mode() {
            horizontal_constraints.push(Constraint::Percentage(20));
        }

        // Split the area into 2 horizontal sections, one for the main app and
        // one for the debug widget in debug mode.
        let horizontal_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(area)
            .to_vec();
        let main_area = horizontal_areas[0];
        if self.state.is_debug_mode() {
            let debug_area = horizontal_areas[1];
            draw_app_widget("Debug", &debug_area, buf, DebugWidget::new(self.state));
        }

        // Define the main area layout
        let main_area_splits = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(10), // Title
                Constraint::Percentage(80), // Other app widgets
                Constraint::Percentage(10), // Controls
            ])
            .split(main_area)
            .to_vec();

        // TITLE
        let title_area = main_area_splits[0];
        draw_app_widget_styled(Block::default(), &title_area, buf, TitleWidget);

        // OTHER APP WIDGETS
        let child_widgets_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_percentages(vec![30, 70]))
            .split(main_area_splits[1])
            .to_vec();

        let channels_area = child_widgets_areas[0];
        draw_app_widget_styled(
            self.get_child_widget_style("Channels", self.state.focussed == FocussedPane::Channels),
            &channels_area,
            buf,
            ChannelsWidget { state: self.state },
        );

        let articles_area = child_widgets_areas[1];
        draw_app_widget_styled(
            self.get_child_widget_style("Articles", self.state.focussed == FocussedPane::Articles),
            &articles_area,
            buf,
            ArticlesWidget { state: self.state },
        );

        // CONTROLS
        let controls_area = main_area_splits[2];
        draw_app_widget_styled(Block::default(), &controls_area, buf, ControlsWidget);
    }
}

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

fn draw_app_widget<T>(title: &'static str, area: &Rect, buffer: &mut Buffer, widget: T)
where
    T: Widget,
{
    let block = Block::default()
        .title_top(Line::from(title).centered())
        .title_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    draw_app_widget_styled(block, area, buffer, widget);
}

fn draw_app_widget_styled<T>(block: Block, area: &Rect, buffer: &mut Buffer, widget: T)
where
    T: Widget,
{
    block.render(*area, buffer);
    let actual_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(Constraint::from_percentages(vec![100]))
        .split(*area)
        .to_vec();
    widget.render(actual_area[0], buffer);
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

impl<'a> Widget for DebugWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();
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

impl<'a> DebugWidget<'a> {
    fn new(state: &'a AppState) -> Self {
        DebugWidget { state }
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
