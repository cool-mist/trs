pub mod actions;
pub mod articles;
pub mod backend;
pub mod channels;
pub mod controls;
pub mod debug;
pub mod title;

use std::{
    io::Stdout,
    sync::mpsc::{channel, Sender},
    time::Duration,
};

use crate::{
    args::{self, UiArgs},
    error::{Result, TrsError},
    persistence::RssChannelD,
};
use articles::ArticlesWidget;
use channels::ChannelsWidget;
use controls::ControlsWidget;
use crossterm::event::{self, KeyEventKind};
use debug::DebugWidget;
use futures::{FutureExt, StreamExt};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding},
};
use title::TitleWidget;
use tokio::sync::mpsc::UnboundedReceiver;

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
    receiver: UnboundedReceiver<Event>,
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
    SyncChannel,
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
    ListChannels(args::ListChannelArgs),
}

/// APP
///  - Listen Event
///  - Publish UiCommandDispatchActions
///
/// BACKEND
///  - Listen UiCommandDispatchActions
///  - Publish BackendEvent
///
/// EVENT LOOP
///  - Listen BackendEvent
///  - Listen crossterm::event::Event
///  - Publish Event
pub async fn ui(args: &UiArgs, db_name: &str) -> Result<()> {
    let (ui_action_publisher, ui_action_receiver) = channel();
    let (backend_event_publisher, backend_event_receiver) = tokio::sync::mpsc::unbounded_channel();
    let event_receiver = start_event_loop(backend_event_receiver);

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
        dispatcher: ui_action_publisher,
        receiver: event_receiver,
    };

    start_backend(db_name, ui_action_receiver, backend_event_publisher);

    app_state
        .dispatcher
        .send(UiCommandDispatchActions::ListChannels(
            args::ListChannelArgs { limit: None },
        ))
        .map_err(|e| TrsError::Error(format!("Unable to send initial app: {}", e)))?;

    let mut terminal = ratatui::init();
    loop {
        draw(&app_state, &mut terminal)?;
        handle_events(&mut app_state).await?;
        if app_state.exit {
            break;
        }
    }

    drop(app_state);
    ratatui::restore();
    Ok(())
}

fn start_backend(
    db_name: &str,
    app_recv: std::sync::mpsc::Receiver<UiCommandDispatchActions>,
    executor_dispatch: tokio::sync::mpsc::UnboundedSender<BackendEvent>,
) {
    let db_name = db_name.to_string();
    std::thread::spawn(move || {
        backend::start(db_name, app_recv, executor_dispatch);
    });
}

fn start_event_loop(
    mut executor_recv: UnboundedReceiver<BackendEvent>,
) -> UnboundedReceiver<Event> {
    let (evt_dispatch, evt_recv) = tokio::sync::mpsc::unbounded_channel();
    let _event_tx = evt_dispatch.clone();
    let _task = tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();
        let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            let tick_delay = tick_interval.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
              user_input = crossterm_event => {
                match user_input {
                  Some(Ok(evt)) => {
                    match evt {
                      crossterm::event::Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                          _event_tx.send(Event::UserInput(crossterm::event::Event::Key(key))).unwrap();
                        }
                      },
                      _ => {}
                    }
                  },
                  _ => {}
                }
              },
              executor_event = executor_recv.recv() => {
                  match executor_event {
                    Some(backend_event) => {
                      _event_tx.send(Event::BackendEvent(backend_event)).unwrap();
                    },
                    None => {}
                  }
              },
              _ = tick_delay => {
                  _event_tx.send(Event::Tick).unwrap();
              },
            }
        }
    });

    evt_recv
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
    BackendEvent(BackendEvent),
    Tick,
}

pub enum BackendEvent {
    ReloadState(Vec<RssChannelD>),
}

async fn handle_events(state: &mut AppState) -> Result<()> {
    let event = state.receiver.recv().await;
    let Some(event) = event else {
        return Ok(());
    };

    match event {
        Event::UserInput(event) => {
            handle_user_input(state, event)?;
        }
        Event::BackendEvent(backend_event) => match backend_event {
            BackendEvent::ReloadState(channels) => {
                state.channels = channels;
                if state.highlighted_channel.is_none() && !state.channels.is_empty() {
                    state.highlighted_channel = Some(0);
                }
            }
        },
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
        let mut horizontal_constraints = vec![Constraint::Fill(5)];
        if is_debug_mode(self.app_state) {
            horizontal_constraints.push(Constraint::Fill(1));
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
                Constraint::Percentage(100), // Channels + Articles
                Constraint::Min(4),          // Controls + Title
            ])
            .split(main_area)
            .to_vec();

        // OTHER APP WIDGETS
        let child_widgets_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_fills([4, 6]))
            .split(main_area_splits[0])
            .to_vec();

        // CHANNELS
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

        // ARTICLES
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

        let controls_title = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_fills([1, 7]))
            .split(main_area_splits[1]);

        // TITLE
        let title_area = controls_title[0];
        draw_app_widget_styled(Block::default(), &title_area, buf, TitleWidget);

        // CONTROLS
        let controls_area = controls_title[1];
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
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .centered();
    if focussed {
        return Block::default().title_top(title).bg(Color::DarkGray);
    }

    Block::default()
        .title_top(title)
        .fg(Color::DarkGray)
        .padding(Padding::uniform(10))
        .border_style(Style::default().fg(Color::DarkGray))
}

fn is_debug_mode(app_state: &AppState) -> bool {
    app_state.debug_enabled && app_state.debug
}
