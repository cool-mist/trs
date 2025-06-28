use std::env;

use rusqlite::{Connection, Statement};

use crate::error::Result;
use crate::error::TrsError;

const CREATE_TABLE: &'static str = "CREATE TABLE IF NOT EXISTS Channels ( \
    id INTEGER PRIMARY KEY, \
    name TEXT NOT NULL, \
    link TEXT NOT NULL UNIQUE, \
    description TEXT \
)";

const ADD_CHANNEL: &'static str = "INSERT INTO Channels (name, link, description) \
          VALUES (?1, ?2, ?3)\
          ON CONFLICT(link) DO UPDATE SET name=?1, description=?3";
const REMOVE_CHANNEL: &'static str = "DELETE FROM Channels WHERE id = ?1";
const LIST_CHANNELS: &'static str = "SELECT id, name, link, description FROM Channels LIMIT ?1";

pub struct Db<'a> {
    pub add_channel: Statement<'a>,
    pub remove_channel: Statement<'a>,
    pub list_channels: Statement<'a>,
}

macro_rules! prepare_sql {
    ($conn:expr, $sql:expr) => {
        $conn.prepare($sql).map_err(|e| {
            TrsError::SqlError(e, format!("Failed to prepare SQL statement: {}", $sql))
        })
    };
}

impl<'a> Db<'a> {
    fn create(conn: &'a Connection) -> Result<Self> {
        let add_channel = prepare_sql!(conn, ADD_CHANNEL)?;
        let remove_channel = prepare_sql!(conn, REMOVE_CHANNEL)?;
        let list_channels = prepare_sql!(conn, LIST_CHANNELS)?;
        Ok(Db {
            add_channel,
            remove_channel,
            list_channels,
        })
    }
}

pub fn init_db(conn: &Connection) -> Result<Db> {
    conn.execute(CREATE_TABLE, ())
        .map_err(|e| TrsError::SqlError(e, "Failed to create Channels table".to_string()))?;
    Db::create(conn)
}

pub fn init_connection() -> Result<Connection> {
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
    conn.execute(CREATE_TABLE, ())
        .map_err(|e| TrsError::SqlError(e, "Failed to create Channels table".to_string()))?;

    Ok(conn)
}
