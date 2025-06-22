use std::env;

use args::{TrsArgs, TrsSubCommand};
use error::TrsError;
use rusqlite::Connection;
pub mod args;
pub mod error;
pub mod parser;

fn main() -> Result<(), TrsError> {
    let args = argh::from_env::<TrsArgs>();

    let conn = init_db()?;
    match args.sub_command {
        TrsSubCommand::AddChannel(add_channel_args) => {
            add_channel(&conn, &add_channel_args.link)?;
        }
        TrsSubCommand::ListChannels(list_channel_args) => {
            list_channels(&conn, list_channel_args.limit)?;
        }
    }

    Ok(())
}

fn add_channel(conn: &Connection, link: &str) -> Result<(), TrsError> {
    let client = reqwest::blocking::Client::new();
    let rss = client.get(link).send().map_err(|e| {
        TrsError::ReqwestError(
            e,
            "Unable to download provided RSS channel link".to_string(),
        )
    })?;

    // TODO: Streaming read
    let bytes = rss.bytes().map_err(|e| {
        TrsError::ReqwestError(e, "Unable to read bytes from RSS response".to_string())
    })?;

    let xml_source_stream = xml::ParserConfig::new()
        .ignore_invalid_encoding_declarations(true)
        .create_reader(&bytes[..]);
    let channel = parser::parse_rss_channel(xml_source_stream)?;
    let mut stmt =
        conn.prepare(
            "INSERT INTO Channels (name, link, description) VALUES (?1, ?2, ?3) ON CONFLICT(link) DO UPDATE SET name=?1, description=?3")?;
    stmt.execute((channel.title, link, channel.description))
        .map_err(|e| TrsError::SqlError(e, "Failed to insert channel into database".to_string()))?;

    Ok(())
}

fn list_channels(conn: &Connection, limit: Option<u32>) -> Result<(), TrsError> {
    let mut stmt = conn.prepare("SELECT id, name, link, description FROM Channels")?;
    let channels_iter = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for row in channels_iter {
        let (id, name, link, description) = row?;
        println!(
            "ID: {}, Name: {}, Link: {}, Description: {}",
            id, name, link, description
        );
    }

    Ok(())
}

fn init_db() -> Result<Connection, TrsError> {
    let home_dir = env::home_dir();
    let db_dir = home_dir
        .map(|dir| dir.join(".config").join("trs"))
        .ok_or(TrsError::Error(
            "Unable to determine home directory".to_string(),
        ))?;

    match std::fs::create_dir_all(&db_dir) {
        Ok(_) => {}
        Err(e) => {
            return Err(TrsError::Error(format!(
                "Failed to create database directory: {}",
                e
            )));
        }
    }

    let db_file = db_dir.join("test.db");
    let conn = Connection::open(db_file)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Channels (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            link  TEXT NOT NULL UNIQUE,
            description  TEXT
        )",
        (),
    )
    .map_err(|e| TrsError::SqlError(e, "Failed to create Channels table".to_string()))?;

    Ok(conn)
}
