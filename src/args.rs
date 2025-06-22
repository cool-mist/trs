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

pub fn valid_url(url: &str) -> Result<String, String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(url.to_string())
    } else {
        Err(format!("Invalid URL: {}", url))
    }
}
