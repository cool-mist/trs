use std::sync::mpsc::Receiver;

use tokio::sync::mpsc::UnboundedSender;

use crate::{commands::TrsEnv, ui::BackendEvent};

use super::UiCommandDispatchActions;

// This one will have to run on the same thread as this manages the sqlite connection
pub fn start(
    db_name: String,
    cmd_recv: Receiver<UiCommandDispatchActions>,
    backend_dispatch: UnboundedSender<BackendEvent>,
) -> () {
    let ctx = TrsEnv::new(db_name.as_str()).unwrap();
    loop {
        let action = cmd_recv.recv();
        let Ok(action) = action else {
            break;
        };

        match action {
            UiCommandDispatchActions::AddChannel(args) => {
                if let Ok(_) = crate::commands::add_channel(&ctx, &args) {
                    send_new_state_default(&ctx, &backend_dispatch);
                };
            }
            UiCommandDispatchActions::RemoveChannel(args) => {
                if let Ok(_) = crate::commands::remove_channel(&ctx, &args) {
                    send_new_state_default(&ctx, &backend_dispatch);
                }
            }
            UiCommandDispatchActions::MarkArticleRead(args) => {
                if let Ok(_) = crate::commands::mark_read(&ctx, &args) {
                    send_new_state_default(&ctx, &backend_dispatch);
                }
            }
            UiCommandDispatchActions::ListChannels(args) => {
                send_new_state(&ctx, args, &backend_dispatch);
            }
        }
    }
}

fn send_new_state_default(
    ctx: &crate::commands::TrsEnv,
    dispatcher: &UnboundedSender<BackendEvent>,
) {
    send_new_state(
        ctx,
        crate::args::ListChannelArgs { limit: None },
        dispatcher,
    );
}

fn send_new_state(
    ctx: &crate::commands::TrsEnv,
    args: crate::args::ListChannelArgs,
    dispatcher: &UnboundedSender<BackendEvent>,
) {
    if let Ok(channels) = crate::commands::list_channels(ctx, &args) {
        dispatcher
            .send(BackendEvent::ReloadState(channels))
            .unwrap_or_default();
    }
}
