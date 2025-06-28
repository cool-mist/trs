use crate::{
    args::{AddChannelArgs, ListChannelArgs, RemoveChannelArgs, TrsArgs, TrsSubCommand},
    error::TrsError,
    parser,
    persistence::Db,
};

pub fn execute(mut db: Db, args: &TrsArgs) -> Result<(), TrsError> {
    let sub_command = &args.sub_command;
    match sub_command {
        TrsSubCommand::AddChannel(add_args) => add_channel(&mut db, add_args),
        TrsSubCommand::ListChannels(list_args) => list_channels(&mut db, list_args),
        TrsSubCommand::RemoveChannel(delete_args) => delete_channel(&mut db, delete_args),
    }
}

fn add_channel(db: &mut Db, args: &AddChannelArgs) -> Result<(), TrsError> {
    let client = reqwest::blocking::Client::new();
    let rss = client.get(&args.link).send().map_err(|e| {
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
    db.add_channel
        .execute((channel.title, &args.link, channel.description))
        .map_err(|e| TrsError::SqlError(e, "Failed to insert channel into database".to_string()))?;

    Ok(())
}

fn list_channels(conn: &mut Db, args: &ListChannelArgs) -> Result<(), TrsError> {
    let channels_iter =
        conn.list_channels
            .query_map([args.limit.unwrap_or_else(|| 999)], |row| {
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

fn delete_channel(db: &mut Db, args: &RemoveChannelArgs) -> Result<(), TrsError> {
    let rows_affected = db
        .remove_channel
        .execute([args.id])
        .map_err(|e| TrsError::SqlError(e, "Failed to delete channel from database".to_string()))?;

    if rows_affected == 0 {
        return Err(TrsError::Error(format!(
            "No channel found with ID: {}",
            args.id
        )));
    }

    println!("Channel with ID {} deleted successfully.", args.id);
    Ok(())
}
