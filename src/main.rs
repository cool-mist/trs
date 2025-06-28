use args::TrsArgs;
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
        ui::ui(terminal)?;
        ratatui::restore();
        return Ok(());
    }

    let args = argh::from_env::<TrsArgs>();
    let conn = persistence::init_connection()?;
    let db = persistence::init_db(&conn)?;
    commands::execute(db, &args).map_err(|e| {
        eprintln!("Error executing command: {}", e);
        e
    })
}
