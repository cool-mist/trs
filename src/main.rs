use args::{TrsArgs, TrsSubCommand};
use commands::TrsEnv;
use error::Result;
pub mod args;
pub mod commands;
pub mod error;
pub mod parser;
pub mod persistence;
pub mod ui;

fn main() -> Result<()> {
    if std::env::args().len() < 2 {
        let terminal = ratatui::init();
        let ctx = TrsEnv::new("test")?;
        ui::ui(ctx, terminal)?;
        ratatui::restore();
        return Ok(());
    }

    let args = argh::from_env::<TrsArgs>();
    let mut ctx = TrsEnv::new("test")?;
    match args.sub_command {
        TrsSubCommand::AddChannel(args) => {
            commands::add_channel(&mut ctx, &args)?;
            Ok(())
        }
        TrsSubCommand::ListChannels(args) => {
            let channels = commands::list_channels(&mut ctx, &args)?;
            for channel in channels {
                println!("{}: {} ({}) updated on {}", channel.id, channel.title, channel.link, channel.last_update);
            }

            Ok(())
        }
        TrsSubCommand::RemoveChannel(args) => commands::remove_channel(&mut ctx, &args),
    }
}
