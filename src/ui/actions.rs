use crate::{
    args,
    error::TrsError,
    persistence::{RssArticleD, RssChannelD},
};

use super::{AppState, FocussedPane, PopupUiAction, UiAction, UiCommandDispatchActions};

pub fn handle_action(
    app_state: &mut AppState,
    event: UiAction,
) -> std::result::Result<(), TrsError> {
    match event {
        UiAction::None => {}
        UiAction::FocusPaneUp => {}
        UiAction::FocusPaneDown => {}
        UiAction::FocusPaneLeft => app_state.focussed = FocussedPane::Channels,
        UiAction::FocusPaneRight => app_state.focussed = FocussedPane::Articles,
        UiAction::ToggleDebug => toggle_debug(app_state),
        UiAction::Exit => app_state.exit = true,
        UiAction::FocusEntryDown => focus_entry_down(app_state),
        UiAction::FocusEntryUp => focus_entry_up(app_state),
        UiAction::OpenArticle => {
            if let Some(channel_idx) = app_state.highlighted_channel {
                if let Some(article_idx) = app_state.highlighted_article {
                    if let Some(channel) = app_state.channels.get_mut(channel_idx) {
                        if let Some(article) = channel.articles.get_mut(article_idx) {
                            article.unread = false;
                            app_state
                                .dispatcher
                                .send(UiCommandDispatchActions::MarkArticleRead(
                                    args::MarkReadArgs {
                                        id: article.id as u32,
                                        unread: false,
                                    },
                                ))
                                .unwrap();
                            _ = open::that(&article.link);
                        }
                    }
                }
            }
        }
        UiAction::ShowAddChannelUi => {
            app_state.show_add_channel_ui = true;
        }
        UiAction::RemoveChannel => {
            let hi_channel = get_highlighted_channel(app_state);
            let Some(channel) = hi_channel else {
                return Ok(());
            };

            let remove_channel_args = args::RemoveChannelArgs {
                id: channel.id as u32,
            };

            app_state
                .dispatcher
                .send(UiCommandDispatchActions::RemoveChannel(remove_channel_args))
                .unwrap();
        }
        UiAction::ToggleReadStatus => {
            let article = get_highlighted_article(app_state);
            let mut article_id = None;
            let mut unread = None;
            if let Some(article) = article {
                unread = Some(!article.unread);
                article_id = Some(article.id);
            }

            if let Some(article_id) = article_id {
                if let Some(unread) = unread {
                    app_state
                        .dispatcher
                        .send(UiCommandDispatchActions::MarkArticleRead(
                            args::MarkReadArgs {
                                id: article_id as u32,
                                unread,
                            },
                        ))
                        .unwrap();
                }
            }
        }
    };
    Ok(())
}

pub fn handle_popup_action(
    state: &mut AppState,
    event: PopupUiAction,
) -> std::result::Result<(), TrsError> {
    match event {
        PopupUiAction::None => {}
        PopupUiAction::Submit => {
            let add_channel_args = args::AddChannelArgs {
                link: state.add_channel.clone(),
            };
            state
                .dispatcher
                .send(UiCommandDispatchActions::AddChannel(add_channel_args))
                .unwrap();
            state.show_add_channel_ui = false;
        }
        PopupUiAction::AddChar(c) => {
            state.add_channel.push(c);
        }
        PopupUiAction::Backspace => {
            state.add_channel.pop();
        }
        PopupUiAction::Close => {
            state.show_add_channel_ui = false;
        }
    };

    Ok(())
}

fn saturating_add(num: usize, to_add: usize, max: usize) -> usize {
    if num + to_add > max {
        max
    } else {
        num + to_add
    }
}

fn min(arg: usize, max_channel_idx: usize) -> usize {
    if arg < max_channel_idx {
        arg
    } else {
        max_channel_idx
    }
}

fn toggle_debug(app_state: &mut AppState) {
    app_state.debug = !app_state.debug;
}

fn get_highlighted_channel<'a>(app_state: &'a AppState) -> Option<&'a RssChannelD> {
    app_state
        .highlighted_channel
        .and_then(|idx| app_state.channels.get(idx).or_else(|| None))
}

fn get_highlighted_article<'a>(app_state: &'a AppState) -> Option<&'a RssArticleD> {
    let hi_article = app_state.highlighted_article?;
    let channel = get_highlighted_channel(app_state)?;
    channel.articles.get(hi_article)
}

fn focus_entry_up(app_state: &mut AppState) {
    match app_state.focussed {
        FocussedPane::Channels => decrement_highlighted_channel_idx(app_state),
        FocussedPane::Articles => decrement_highlighted_article_idx(app_state),
    }
    .unwrap_or(false);
}

fn focus_entry_down(app_state: &mut AppState) {
    match app_state.focussed {
        FocussedPane::Channels => increment_highlighted_channel_idx(app_state),
        FocussedPane::Articles => increment_highlighted_article_idx(app_state),
    }
    .unwrap_or(false);
}

fn increment_highlighted_channel_idx(app_state: &mut AppState) -> Option<bool> {
    let channels_len = app_state.channels.len();
    if channels_len == 0 {
        app_state.highlighted_channel = None;
        return Some(false);
    }

    let max_channel_idx = channels_len.saturating_sub(1);
    app_state.highlighted_channel = app_state
        .highlighted_channel
        .map(|idx| saturating_add(idx, 1, max_channel_idx))
        .or_else(|| Some(0));

    update_highligted_article(app_state)
}

fn decrement_highlighted_channel_idx(app_state: &mut AppState) -> Option<bool> {
    app_state.highlighted_channel = app_state
        .highlighted_channel
        .map(|idx| idx.saturating_sub(1));

    update_highligted_article(app_state)
}

// When changing the channel, update the idx of the article to be within the range
fn update_highligted_article(app_state: &mut AppState) -> Option<bool> {
    let hi_channel_articles_max = get_highlighted_channel(app_state)?.articles.len();
    let hi_article_idx = app_state.highlighted_article?;
    let max_article_idx = hi_channel_articles_max.saturating_sub(1);

    if max_article_idx == 0 {
        app_state.highlighted_article = None;
    } else if hi_article_idx > max_article_idx {
        app_state.highlighted_article = Some(max_article_idx);
    } else {
        app_state.highlighted_article = Some(hi_article_idx);
    }

    Some(true)
}

fn increment_highlighted_article_idx(app_state: &mut AppState) -> Option<bool> {
    let hi_article_max_idx = get_highlighted_channel(app_state)?
        .articles
        .len()
        .saturating_sub(1);

    app_state.highlighted_article = app_state
        .highlighted_article
        .map(|idx| saturating_add(idx, 1, hi_article_max_idx))
        .or_else(|| Some(min(0, hi_article_max_idx)));

    Some(true)
}

fn decrement_highlighted_article_idx(app_state: &mut AppState) -> Option<bool> {
    app_state.highlighted_article = app_state
        .highlighted_article
        .map(|idx| idx.saturating_sub(1));

    Some(true)
}
