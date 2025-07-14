use std::env;

use rusqlite::Connection;
use time::OffsetDateTime;

use crate::error::Result;
use crate::error::TrsError;
use crate::parser::RssArticle;
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
    pub_date TIMESTAMP, \
    last_update TIMESTAMP DEFAULT CURRENT_TIMESTAMP, \
    unread BOOLEAN DEFAULT FALSE, \
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

const ADD_ARTICLE: &'static str =
    "INSERT INTO Articles (channel_id, title, description, link, pub_date, last_update) \
          VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP) \
          ON CONFLICT(link) DO UPDATE SET pub_date=?5, last_update=CURRENT_TIMESTAMP";

const GET_ARTICLES_BY_CHANNEL: &'static str =
    "SELECT id, channel_id, title, description, link, pub_date, last_update, unread FROM Articles WHERE channel_id = ?1";

const GET_ARTICLE: &'static str =
    "SELECT id, channel_id, title, description, link, pub_date, last_update, unread FROM Articles WHERE link = ?1";

const LIST_ARTICLES: &'static str =
    "SELECT id, channel_id, title, description, link, pub_date, last_update, unread FROM Articles";

const MARK_ARTICLE_READ: &'static str = "UPDATE Articles SET unread = FALSE WHERE id = ?1";

const MARK_ARTICLE_UNREAD: &'static str = "UPDATE Articles SET unread = TRUE WHERE id = ?1";

pub struct Db {
    connection: Connection,
}

pub struct RssChannelD {
    pub id: i64,
    pub title: String,
    pub link: String,
    pub description: String,
    pub last_update: OffsetDateTime,
    pub articles: Vec<RssArticleD>,
}

pub struct RssArticleD {
    pub id: i64,
    pub channel_id: i64,
    pub title: String,
    pub description: String,
    pub link: String,
    pub pub_date: Option<OffsetDateTime>,
    pub last_update: Option<OffsetDateTime>,
    pub unread: bool,
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
        let mut channel = self
            .connection
            .query_row(GET_CHANNEL, (link,), Db::map_rsschanneld)
            .map_err(|e| {
                TrsError::SqlError(e, format!("Failed to retrieve channel with link {}", link))
            })?;

        channel.articles = self.list_articles_by_channel(channel.id)?;
        Ok(channel)
    }

    pub fn add_channel(&self, channel: &RssChannel) -> Result<RssChannelD> {
        self.connection
            .execute(
                ADD_CHANNEL,
                (&channel.title, &channel.link, &channel.description),
            )
            .map_err(|e| TrsError::SqlError(e, "Failed to add channel".to_string()))?;

        let mut inserted_channel = self.get_channel(&channel.link).map_err(|e| {
            TrsError::Error(format!("Failed to retrieve channel after adding: {}", e))
        })?;

        let mut articles = Vec::new();
        for article in &channel.articles {
            let article = self.add_article(inserted_channel.id, article)?;
            articles.push(article);
        }

        inserted_channel.articles = articles;
        Ok(inserted_channel)
    }

    pub fn remove_channel(&self, id: u32) -> Result<usize> {
        self.connection
            .execute(REMOVE_CHANNEL, (id,))
            .map(|rows| rows as usize)
            .map_err(|e| TrsError::SqlError(e, "Failed to remove channel".to_string()))
    }

    pub fn list_channels(&self, limit: u32) -> Result<Vec<RssChannelD>> {
        let mut channels = self
            .connection
            .prepare(LIST_CHANNELS)
            .map_err(|e| TrsError::SqlError(e, "Failed to prepare query".to_string()))?
            .query_map((limit as i64,), Db::map_rsschanneld)
            .map_err(|e| TrsError::SqlError(e, "Failed to list channels".to_string()))?
            .map(|r| r.unwrap())
            .collect::<Vec<RssChannelD>>();

        let mut articles = self.list_articles()?;
        for channel in &mut channels {
            for article in &mut articles {
                if article.channel_id == channel.id {
                    let copied = std::mem::replace(article, RssArticleD::dummy());
                    channel.articles.push(copied);
                }
            }
        }

        Ok(channels)
    }

    pub fn mark_article_read(&self, id: i64) -> Result<usize> {
        self.connection
            .execute(MARK_ARTICLE_READ, (id,))
            .map(|rows| rows as usize)
            .map_err(|e| TrsError::SqlError(e, "Failed to mark article as read".to_string()))
    }

    pub fn mark_article_unread(&self, id: i64) -> Result<usize> {
        self.connection
            .execute(MARK_ARTICLE_UNREAD, (id,))
            .map(|rows| rows as usize)
            .map_err(|e| TrsError::SqlError(e, "Failed to mark article as unread".to_string()))
    }

    fn add_article(&self, channel_id: i64, article: &RssArticle) -> Result<RssArticleD> {
        self.connection
            .execute(
                ADD_ARTICLE,
                (
                    channel_id,
                    &article.title,
                    &article.description,
                    &article.link,
                    article.date.map(|d| d.to_string()),
                ),
            )
            .map_err(|e| TrsError::SqlError(e, "Failed to add article".to_string()))?;

        self.get_article(&article.link)
            .map_err(|e| TrsError::Error(format!("Failed to retrieve article after adding: {}", e)))
    }

    fn get_article(&self, link: &str) -> Result<RssArticleD> {
        self.connection
            .query_row(GET_ARTICLE, (link,), Db::map_rssarticled)
            .map_err(|e| {
                TrsError::SqlError(e, format!("Failed to retrieve article with link {}", link))
            })
    }

    fn list_articles_by_channel(&self, channel_id: i64) -> Result<Vec<RssArticleD>> {
        let articles = self
            .connection
            .prepare(GET_ARTICLES_BY_CHANNEL)
            .map_err(|e| TrsError::SqlError(e, "Failed to prepare query".to_string()))?
            .query_map((channel_id,), Db::map_rssarticled)
            .map_err(|e| TrsError::SqlError(e, "Failed to list articles".to_string()))?
            .map(|r| r.unwrap())
            .collect::<Vec<RssArticleD>>();

        Ok(articles)
    }

    fn list_articles(&self) -> Result<Vec<RssArticleD>> {
        let articles = self
            .connection
            .prepare(LIST_ARTICLES)
            .map_err(|e| TrsError::SqlError(e, "Failed to prepare query".to_string()))?
            .query_map([], Db::map_rssarticled)
            .map_err(|e| TrsError::SqlError(e, "Failed to list articles".to_string()))?
            .map(|r| r.unwrap())
            .collect::<Vec<RssArticleD>>();

        Ok(articles)
    }

    fn map_rsschanneld(row: &rusqlite::Row) -> std::result::Result<RssChannelD, rusqlite::Error> {
        Ok(RssChannelD::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            Vec::new(),
        ))
    }

    fn map_rssarticled(row: &rusqlite::Row) -> std::result::Result<RssArticleD, rusqlite::Error> {
        Ok(RssArticleD {
            id: row.get(0)?,
            channel_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            link: row.get(4)?,
            pub_date: row.get(5).ok(),
            last_update: row.get(6).ok(),
            unread: row.get(7)?,
        })
    }
}

impl RssChannelD {
    fn new(
        id: i64,
        title: String,
        link: String,
        description: String,
        last_update: OffsetDateTime,
        articles: Vec<RssArticleD>,
    ) -> Self {
        RssChannelD {
            id,
            title,
            link,
            description,
            last_update,
            articles,
        }
    }
}

impl RssArticleD {
    fn dummy() -> Self {
        RssArticleD {
            id: -1,
            channel_id: 0,
            title: String::new(),
            description: String::new(),
            link: String::new(),
            pub_date: None,
            last_update: None,
            unread: false,
        }
    }
}
