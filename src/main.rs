use args::{TrsArgs, TrsSubCommand};
use commands::TrsEnv;
use error::Result;
pub mod args;
pub mod commands;
pub mod error;
pub mod parser;
pub mod persistence;
pub mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let args = argh::from_env::<TrsArgs>();
    let db_name = "test4";
    match args.sub_command {
        TrsSubCommand::AddChannel(args) => {
            let mut ctx = TrsEnv::new(db_name)?;
            commands::add_channel(&mut ctx, &args)?;
            Ok(())
        }
        TrsSubCommand::ListChannels(args) => {
            let mut ctx = TrsEnv::new(db_name)?;
            let channels = commands::list_channels(&mut ctx, &args)?;
            for channel in channels {
                println!(
                    "{}: {} ({}) updated on {}",
                    channel.id, channel.title, channel.link, channel.last_update
                );
            }

            Ok(())
        }
        TrsSubCommand::RemoveChannel(args) => {
            let mut ctx = TrsEnv::new("test3")?;
            commands::remove_channel(&mut ctx, &args)
        }
        TrsSubCommand::GetArticles(args) => {
            let mut ctx = TrsEnv::new(db_name)?;
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
        TrsSubCommand::MarkRead(args) => {
            let mut ctx = TrsEnv::new("test3")?;
            commands::mark_read(&mut ctx, &args)
        }
        TrsSubCommand::Ui(args) => ui::ui(&args, db_name).await,
    }
}
