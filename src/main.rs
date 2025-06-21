use error::TrsError;
use xml::ParserConfig;
pub mod error;
pub mod parser;

fn main() -> Result<(), TrsError> {
    let bytes = include_bytes!("../sample/rss.xml");
    let xml_source_stream = ParserConfig::new()
        .ignore_invalid_encoding_declarations(true)
        .create_reader(&bytes[..]);
    let rss_channel = parser::parse_rss_channel(xml_source_stream)?;

    println!("{}", rss_channel.title);
    println!("{}", rss_channel.link);
    println!("{}", rss_channel.description);
    for article in &rss_channel.articles {
        let max_title_chars = article.title.len().min(47);
        let max_link_chars = article.link.len().min(67);
        println!(
            "| {} | {:.<50} | {:.<70} |",
            article.date,
            &article.title[0..max_title_chars],
            &article.link[0..max_link_chars]
        );
    }

    println!(
        "There are {} articles in the channel.",
        rss_channel.articles.len()
    );

    Ok(())
}
