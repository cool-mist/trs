use std::sync::mpsc::{Receiver, Sender};

use super::UiCommandDispatchActions;

pub struct UiCommandExecutor {
    pub command_receiver: Receiver<UiCommandDispatchActions>,
    pub status_sender: Sender<u64>,
}

impl UiCommandExecutor {
    pub fn new(
        command_receiver: Receiver<UiCommandDispatchActions>,
        status_sender: Sender<u64>,
    ) -> Self {
        UiCommandExecutor {
            command_receiver,
            status_sender,
        }
    }

    pub fn run(&self, ctx: crate::commands::TrsEnv) -> () {
        loop {
            let action = self.command_receiver.recv();
            let Ok(action) = action else {
                break;
            };

            match action {
                UiCommandDispatchActions::AddChannel(args) => {
                    if let Ok(_) = crate::commands::add_channel(&ctx, &args) {
                        self.status_sender.send(1).unwrap_or_default();
                    };
                }

                UiCommandDispatchActions::RemoveChannel(args) => {
                    if let Ok(_) = crate::commands::remove_channel(&ctx, &args) {
                        self.status_sender.send(1).unwrap_or_default();
                    }
                }

                UiCommandDispatchActions::MarkArticleRead(args) => {
                    if let Ok(_) = crate::commands::mark_read(&ctx, &args) {
                        self.status_sender.send(1).unwrap_or_default();
                    }
                }
            }
        }
    }
}
