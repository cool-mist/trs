use args::{TrsArgs, TrsSubCommand};
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
        let conn = persistence::init_connection()?;
        let db = persistence::init_db(&conn)?;
        ui::ui(db, terminal)?;
        ratatui::restore();
        return Ok(());
    }

    let args = argh::from_env::<TrsArgs>();
    let conn = persistence::init_connection()?;
    let mut db = persistence::init_db(&conn)?;
    match args.sub_command {
        TrsSubCommand::AddChannel(args) => commands::add_channel(&mut db, &args),
        TrsSubCommand::ListChannels(args) => {
            let channels = commands::list_channels(&mut db, &args)?;
            for channel in channels {
                println!("{}: {} ({})", channel.id, channel.title, channel.link);
            }

            return Ok(());
        }
        TrsSubCommand::RemoveChannel(args) => commands::remove_channel(&mut db, &args),
    }
}
