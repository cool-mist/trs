use argh::FromArgs;

/// Tiny RSS reader
#[derive(FromArgs, PartialEq, Debug)]
pub struct TrsArgs {
    #[argh(subcommand)]
    pub sub_command: TrsSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum TrsSubCommand {
    AddChannel(AddChannelArgs),
    ListChannels(ListChannelArgs),
    GetArticles(GetArticlesArgs),
    RemoveChannel(RemoveChannelArgs),
    MarkRead(MarkReadArgs),
    Ui(UiArgs),
}

/// Add a new RSS channel
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "add")]
pub struct AddChannelArgs {
    /// link to RSS channel
    #[argh(option, from_str_fn(valid_url))]
    pub link: String,
}

/// List RSS channels
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
pub struct ListChannelArgs {
    /// limit the number of channels to list
    #[argh(option)]
    pub limit: Option<u32>,
}

/// Get articles
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "articles")]
pub struct GetArticlesArgs {
    /// id of the channel to get articles from
    #[argh(option, short = 'c')]
    pub channel_id: Option<u32>,

    /// only get unread articles
    #[argh(switch)]
    pub unread: bool,
}

/// Mark article as read/unread
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "read")]
pub struct MarkReadArgs {
    /// id of the article to mark read/unread
    #[argh(option)]
    pub id: u32,

    /// mark the article as unread
    #[argh(switch)]
    pub unread: bool,
}

/// Delete an RSS channel
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "remove")]
pub struct RemoveChannelArgs {
    /// delete the channel with this id
    #[argh(option)]
    pub id: u32,
}

/// Open UI
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ui")]
pub struct UiArgs {
    /// enable debug window
    #[argh(switch)]
    pub debug: bool,
}

pub fn valid_url(url: &str) -> Result<String, String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(url.to_string())
    } else {
        Err(format!("Invalid URL: {}", url))
    }
}
