use crate::{
    args::{self, AddChannelArgs, ListChannelArgs, RemoveChannelArgs},
    error::TrsError,
    parser,
    persistence::{Db, RssChannelD},
};

pub struct TrsEnv {
    name: String,
    db: Db,
    http_client: reqwest::blocking::Client,
}

impl Clone for TrsEnv {
    fn clone(&self) -> Self {
        let name = self.name.clone();
        TrsEnv::new(&name).expect("Failed to clone TrsEnv")
    }
}

impl TrsEnv {
    pub fn new(instance_name: &str) -> Result<Self, TrsError> {
        let db = Db::create(instance_name)?;
        let http_client = reqwest::blocking::Client::builder()
            .user_agent("cool-mist/trs")
            .build()
            .map_err(|e| TrsError::ReqwestError(e, "Failed to create HTTP client".to_string()))?;
        Ok(TrsEnv {
            name: instance_name.to_string(),
            db,
            http_client,
        })
    }
}

pub fn add_channel(ctx: &TrsEnv, args: &AddChannelArgs) -> Result<RssChannelD, TrsError> {
    let rss = ctx.http_client.get(&args.link).send().map_err(|e| {
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
    ctx.db.add_channel(&channel)
}

pub fn list_channels(ctx: &TrsEnv, args: &ListChannelArgs) -> Result<Vec<RssChannelD>, TrsError> {
    ctx.db.list_channels(args.limit.unwrap_or(u32::MAX))
}

pub fn remove_channel(ctx: &TrsEnv, args: &RemoveChannelArgs) -> Result<(), TrsError> {
    ctx.db.remove_channel(args.id).map(|_| ())
}

pub fn mark_read(ctx: &TrsEnv, args: &args::MarkReadArgs) -> Result<(), TrsError> {
    match args.unread {
        true => ctx.db.mark_article_unread(args.id as i64)?,
        false => ctx.db.mark_article_read(args.id as i64)?,
    };

    Ok(())
}

pub fn get_articles_by_channel(
    ctx: &mut TrsEnv,
    args: &args::GetArticlesArgs,
) -> Result<Vec<RssChannelD>, TrsError> {
    let mut channels = ctx.db.list_channels(u32::MAX)?;
    channels.retain(|channel| {
        if let Some(channel_id) = args.channel_id {
            channel_id as i64 == channel.id
        } else {
            true
        }
    });

    if args.unread {
        for channel in &mut channels {
            channel.articles.retain(|article| article.unread);
        }
    }

    Ok(channels)
}
