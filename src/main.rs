use error::TrsError;
use xml::ParserConfig;
pub mod error;
pub mod parser;

fn main() -> Result<(), TrsError> {
    let bytes = include_bytes!("../sample/rss2.xml");
    let xml_source_stream = ParserConfig::new()
        .ignore_invalid_encoding_declarations(true)
        .create_reader(&bytes[..]);
    let rss_channel = parser::parse_rss_channel(xml_source_stream)?;

    println!("{}", rss_channel.title);
    println!("{}", rss_channel.link);
    println!("{}", rss_channel.description);
    for article in &rss_channel.articles {
        println!("{} {:^50} {:<}", article.date, article.title, article.link);
    }

    println!(
        "There are {} articles in the channel.",
        rss_channel.articles.len()
    );

    Ok(())
}
