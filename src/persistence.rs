use std::env;

use rusqlite::Connection;
use time::OffsetDateTime;

use crate::error::Result;
use crate::error::TrsError;
use crate::parser::RssChannel;

const SCHEMA_CHANNELS: &'static str = "CREATE TABLE IF NOT EXISTS Channels ( \
    id INTEGER PRIMARY KEY, \
    name TEXT NOT NULL, \
    link TEXT NOT NULL UNIQUE, \
    description TEXT, \
    last_update TIMESTAMP DEFAULT CURRENT_TIMESTAMP \
)";

const SCHEMA_ARTICLES: &'static str = "CREATE TABLE IF NOT EXISTS Articles ( \
    id INTEGER PRIMARY KEY, \
    channel_id INTEGER NOT NULL, \
    title TEXT NOT NULL, \
    description TEXT, \
    link TEXT NOT NULL UNIQUE, \
    pub TIMESTAMP, \
    last_update TIMESTAMP DEFAULT CURRENT_TIMESTAMP, \
    unread BOOLEAN DEFAULT TRUE, \
    FOREIGN KEY(channel_id) REFERENCES Channels(id) ON DELETE CASCADE \
)";

const ADD_CHANNEL: &'static str = "INSERT INTO Channels (name, link, description, last_update) \
          VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)\
          ON CONFLICT(link) DO UPDATE SET name=?1, description=?3, last_update=CURRENT_TIMESTAMP";
const REMOVE_CHANNEL: &'static str = "DELETE FROM Channels WHERE id = ?1";
const LIST_CHANNELS: &'static str =
    "SELECT id, name, link, description, last_update FROM Channels LIMIT ?1";
const GET_CHANNEL: &'static str =
    "SELECT id, name, link, description, last_update FROM Channels WHERE link = ?1";

const ADD_ARTICLE: &'static str = "INSERT INTO Articles (channel_id, title, description, link, pub, last_update) \
          VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP) \
          ON CONFLICT(link) DO UPDATE SET pub=?5, last_update=CURRENT_TIMESTAMP";

const GET_ARTICLES_BY_CHANNEL: &'static str =
    "SELECT id, channel_id, title, description, link, pub, last_update, unread FROM Articles WHERE channel_id = ?1";

const LIST_ARTICLES: &'static str =
    "SELECT id, channel_id, title, description, link, pub, last_update, unread FROM Articles ORDER BY pub DESC LIMIT ?2";

const MARK_ARTICLE_READ: &'static str =
    "UPDATE Articles SET unread = FALSE WHERE id = ?1";

const MARK_ARTICLE_UNREAD: &'static str =
    "UPDATE Articles SET unread = TRUE WHERE id = ?1";

pub struct Db {
    connection: Connection,
}

pub struct RssChannelD {
    pub id: i64,
    pub title: String,
    pub link: String,
    pub description: String,
    pub last_update: OffsetDateTime,
}

macro_rules! schema_sql {
    ($conn:expr, $sql:expr) => {
        $conn
            .execute($sql, ())
            .map_err(|e| TrsError::SqlError(e, "Failed to create Channels table".to_string()))?;
    };
}

impl Db {
    pub fn create(instance_name: &str) -> Result<Self> {
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

        let db_file = db_dir.join(format!("{}.db", instance_name));
        let connection = Connection::open(db_file)?;

        schema_sql!(connection, SCHEMA_CHANNELS);
        schema_sql!(connection, SCHEMA_ARTICLES);
        Ok(Db { connection })
    }

    pub fn get_channel(&self, link: &str) -> Result<RssChannelD> {
        self.connection
            .query_row(GET_CHANNEL, (link,), Db::map_rsschanneld)
            .map_err(|e| {
                TrsError::SqlError(e, format!("Failed to retrieve channel with link {}", link))
            })
    }

    pub fn add_channel(&self, channel: &RssChannel) -> Result<RssChannelD> {
        self.connection
            .execute(
                ADD_CHANNEL,
                (&channel.title, &channel.link, &channel.description),
            )
            .map_err(|e| TrsError::SqlError(e, "Failed to add channel".to_string()))?;

        let inserted_channel = self.get_channel(&channel.link)
            .map_err(|e| TrsError::Error(format!("Failed to retrieve channel after adding: {}", e)))?;

        for article in &channel.articles {
            self.connection
                .execute(
                    ADD_ARTICLE,
                    (
                        &inserted_channel.id,
                        &article.title,
                        &article.description,
                        &article.link,
                        article.date.map(|d| d.to_string()),
                    ),
                )
                .map_err(|e| TrsError::SqlError(e, "Failed to add article".to_string()))?;
        }

        Ok(inserted_channel)
    }

    pub fn remove_channel(&self, id: u32) -> Result<usize> {
        self.connection
            .execute(REMOVE_CHANNEL, (id,))
            .map(|rows| rows as usize)
            .map_err(|e| TrsError::SqlError(e, "Failed to remove channel".to_string()))
    }

    pub fn list_channels(&self, limit: u32) -> Result<Vec<RssChannelD>> {
        let channels = self
            .connection
            .prepare(LIST_CHANNELS)
            .map_err(|e| TrsError::SqlError(e, "Failed to prepare query".to_string()))?
            .query_map((limit as i64,), Db::map_rsschanneld)
            .map_err(|e| TrsError::SqlError(e, "Failed to list channels".to_string()))?
            .map(|r| r.unwrap())
            .collect::<Vec<RssChannelD>>();

        Ok(channels)
    }

    fn map_rsschanneld(row: &rusqlite::Row) -> std::result::Result<RssChannelD, rusqlite::Error> {
        Ok(RssChannelD::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    }
}

impl RssChannelD {
    fn new(
        id: i64,
        title: String,
        link: String,
        description: String,
        last_update: OffsetDateTime,
    ) -> Self {
        RssChannelD {
            id,
            title,
            link,
            description,
            last_update,
        }
    }
}
