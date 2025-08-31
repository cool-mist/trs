pub mod actions;
pub mod articles;
pub mod channels;
pub mod controls;
pub mod debug;
pub mod executor;
pub mod title;

use std::{
    io::Stdout,
    sync::mpsc::{channel, Receiver, Sender},
    thread, time::Duration,
};

use crate::{
    args::{self, ListChannelArgs, UiArgs},
    commands::{self, TrsEnv},
    error::{Result, TrsError},
    persistence::RssChannelD,
};
use articles::ArticlesWidget;
use channels::ChannelsWidget;
use controls::ControlsWidget;
use crossterm::event;
use debug::DebugWidget;
use executor::UiCommandExecutor;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use title::TitleWidget;

pub struct AppState {
    exit: bool,
    debug_enabled: bool,
    debug: bool,
    channels: Vec<RssChannelD>,
    focussed: FocussedPane,
    highlighted_channel: Option<usize>,
    highlighted_article: Option<usize>,
    last_action: Option<UiAction>,
    show_add_channel_ui: bool,
    add_channel: String,
    dispatcher: Sender<UiCommandDispatchActions>,
    receiver: Receiver<u64>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FocussedPane {
    Channels,
    Articles,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    None,
    FocusPaneRight,
    FocusPaneLeft,
    FocusPaneUp,
    FocusPaneDown,
    FocusEntryUp,
    FocusEntryDown,
    ToggleDebug,
    OpenArticle,
    ShowAddChannelUi,
    RemoveChannel,
    ToggleReadStatus,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupUiAction {
    None,
    Submit,
    AddChar(char),
    Backspace,
    Close,
}

#[derive(Debug)]
pub enum UiCommandDispatchActions {
    AddChannel(args::AddChannelArgs),
    RemoveChannel(args::RemoveChannelArgs),
    MarkArticleRead(args::MarkReadArgs),
}

pub fn ui(ctx: TrsEnv, args: &UiArgs) -> Result<()> {
    let (tdispatch, rdispatch) = channel();
    let (tupdate, rupdate) = channel();
    let mut terminal = ratatui::init();
    let mut app_state = AppState {
        channels: Vec::new(),
        exit: false,
        debug_enabled: args.debug,
        debug: false,
        focussed: FocussedPane::Channels,
        highlighted_article: None,
        highlighted_channel: None,
        last_action: None,
        show_add_channel_ui: false,
        add_channel: String::new(),
        dispatcher: tdispatch,
        receiver: rupdate,
    };

    let ctx_cloned = ctx.clone();
    let executor = UiCommandExecutor::new(rdispatch, tupdate);
    let executor_handle = thread::spawn(move || {
        executor.run(ctx_cloned);
    });

    let channels = commands::list_channels(&ctx, &ListChannelArgs { limit: None })?;
    app_state.channels = channels;

    loop {
        draw(&app_state, &mut terminal)?;
        handle_events(&mut app_state, &ctx)?;
        if app_state.exit {
            break;
        }
    }

    drop(app_state);
    executor_handle.join().unwrap();
    ratatui::restore();
    Ok(())
}

fn draw(app_state: &AppState, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal
        .draw(|f| {
            f.render_widget(AppStateWidget::new(app_state), f.area());
        })
        .map_err(|e| TrsError::TuiError(e))?;

    Ok(())
}

pub enum Event {
    UserInput(crossterm::event::Event),
    ReloadState,
    Tick,
}

fn handle_events(state: &mut AppState, ctx: &TrsEnv) -> Result<()> {
    let event = get_event(state)?;
    match event {
        Event::UserInput(event) => {
            handle_user_input(state, event)?;
        }
        Event::ReloadState => {
            let channels = commands::list_channels(&ctx, &ListChannelArgs { limit: None })?;
            state.channels = channels;
        }
        Event::Tick => {}
    };

    Ok(())
}

fn handle_user_input(state: &mut AppState, event: event::Event) -> Result<()> {
    if state.show_add_channel_ui {
        let popup_ui_action = controls::parse_popup_ui_action(event);
        actions::handle_popup_action(state, popup_ui_action)?;
        return Ok(());
    }

    let ui_action = controls::parse_ui_action(event);
    state.last_action = Some(ui_action.clone());
    actions::handle_action(state, ui_action)?;
    return Ok(());
}

fn get_event(state: &mut AppState) -> Result<Event> {
    let recv_action = state.receiver.try_recv();
    if let Ok(_) = recv_action {
        return Ok(Event::ReloadState);
    }

    let raw_event = event::poll(Duration::from_millis(250)).map_err(|e| TrsError::TuiError(e))?;
    if raw_event == false {
        return Ok(Event::Tick);
    }

    // It's guaranteed that an event is available now
    Ok(Event::UserInput(event::read().unwrap()))
}

struct AppStateWidget<'a> {
    app_state: &'a AppState,
}

impl<'a> AppStateWidget<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }
}

impl<'a> Widget for AppStateWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut horizontal_constraints = vec![Constraint::Percentage(100)];
        if is_debug_mode(self.app_state) {
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
        if is_debug_mode(self.app_state) {
            let debug_area = horizontal_areas[1];
            draw_app_widget("Debug", &debug_area, buf, DebugWidget::new(self.app_state));
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
            get_child_widget_style(
                "Channels",
                self.app_state.focussed == FocussedPane::Channels,
            ),
            &channels_area,
            buf,
            ChannelsWidget::new(self.app_state),
        );

        let articles_area = child_widgets_areas[1];
        draw_app_widget_styled(
            get_child_widget_style(
                "Articles",
                self.app_state.focussed == FocussedPane::Articles,
            ),
            &articles_area,
            buf,
            ArticlesWidget::new(self.app_state),
        );

        // CONTROLS
        let controls_area = main_area_splits[2];
        draw_app_widget_styled(Block::default(), &controls_area, buf, ControlsWidget);
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

fn get_child_widget_style<'a>(arg: &'a str, focussed: bool) -> Block<'a> {
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

fn is_debug_mode(app_state: &AppState) -> bool {
    app_state.debug_enabled && app_state.debug
}
