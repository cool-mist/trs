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
                println!(
                    "{}: {} ({}) updated on {}",
                    channel.id, channel.title, channel.link, channel.last_update
                );
            }

            Ok(())
        }
        TrsSubCommand::RemoveChannel(args) => commands::remove_channel(&mut ctx, &args),
        TrsSubCommand::GetArticles(args) => {
            let channels = commands::get_articles_by_channel(&mut ctx, &args)?;
            for channel in channels {
                println!(
                    "Channel #{}: {} ({})",
                    channel.id, channel.title, channel.link
                );
                for article in channel.articles {
                    println!(
                        " #{} - {} ({}) [{}]",
                        article.id,
                        article.title,
                        article.link,
                        article
                            .pub_date
                            .map_or("No date".to_string(), |d| d.to_string())
                    );
                }
            }
            Ok(())
        }
        TrsSubCommand::MarkRead(args) => commands::mark_read(&mut ctx, &args),
        TrsSubCommand::Ui(args) => ui::ui(&mut ctx, &args),
    }
}
