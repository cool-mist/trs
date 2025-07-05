use crate::{
    args::{AddChannelArgs, ListChannelArgs, RemoveChannelArgs},
    error::TrsError,
    parser,
    persistence::{Db, RssChannelD},
};

pub struct TrsEnv {
    db: Db,
    http_client: reqwest::blocking::Client,
}

impl TrsEnv {
    pub fn new(instance_name: &str) -> Result<Self, TrsError> {
        let db = Db::create(instance_name)?;
        let http_client = reqwest::blocking::Client::builder()
            .user_agent("cool-mist/trs")
            .build()
            .map_err(|e| TrsError::ReqwestError(e, "Failed to create HTTP client".to_string()))?;
        Ok(TrsEnv { db, http_client })
    }
}

pub fn add_channel(ctx: &mut TrsEnv, args: &AddChannelArgs) -> Result<RssChannelD, TrsError> {
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

pub fn list_channels(
    ctx: &mut TrsEnv,
    args: &ListChannelArgs,
) -> Result<Vec<RssChannelD>, TrsError> {
    ctx.db.list_channels(args.limit.unwrap_or(u32::MAX))
}

pub fn remove_channel(ctx: &mut TrsEnv, args: &RemoveChannelArgs) -> Result<(), TrsError> {
    ctx.db.remove_channel(args.id).map(|_| ())
}
