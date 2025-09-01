use std::sync::mpsc::Receiver;

use tokio::sync::mpsc::UnboundedSender;

use crate::{commands::TrsEnv, ui::BackendEvent};

use super::UiCommandDispatchActions;

pub struct UiCommandExecutor {
    pub app_recv: Receiver<UiCommandDispatchActions>,
    pub executor_dispatch: UnboundedSender<BackendEvent>,
}

impl UiCommandExecutor {
    pub fn new(
        app_recv: Receiver<UiCommandDispatchActions>,
        executor_dispatch: UnboundedSender<BackendEvent>,
    ) -> Self {
        UiCommandExecutor {
            app_recv,
            executor_dispatch,
        }
    }

    // This one will have to run on the same thread as this manages the sqlite connection
    pub fn run(&mut self, db_name: String) -> () {
        let ctx = TrsEnv::new(db_name.as_str()).unwrap();
        loop {
            let action = self.app_recv.recv();
            let Ok(action) = action else {
                break;
            };

            match action {
                UiCommandDispatchActions::AddChannel(args) => {
                    if let Ok(_) = crate::commands::add_channel(&ctx, &args) {
                        self.send_new_state_default(&ctx);
                    };
                }
                UiCommandDispatchActions::RemoveChannel(args) => {
                    if let Ok(_) = crate::commands::remove_channel(&ctx, &args) {
                        self.send_new_state_default(&ctx);
                    }
                }
                UiCommandDispatchActions::MarkArticleRead(args) => {
                    if let Ok(_) = crate::commands::mark_read(&ctx, &args) {
                        self.send_new_state_default(&ctx);
                    }
                }
                UiCommandDispatchActions::ListChannels(args) => {
                    self.send_new_state(&ctx, args);
                }
            }
        }
    }

    fn send_new_state_default(&mut self, ctx: &crate::commands::TrsEnv) {
        self.send_new_state(ctx, crate::args::ListChannelArgs { limit: None });
    }

    fn send_new_state(
        &mut self,
        ctx: &crate::commands::TrsEnv,
        args: crate::args::ListChannelArgs,
    ) {
        if let Ok(channels) = crate::commands::list_channels(ctx, &args) {
            self.executor_dispatch
                .send(BackendEvent::ReloadState(channels))
                .unwrap_or_default();
        }
    }
}
