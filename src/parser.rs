use std::io::Read;

use time::format_description;
use time::format_description::well_known::Iso8601;
use time::format_description::well_known::Rfc2822;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use time::PrimitiveDateTime;
use xml::{reader::XmlEvent, EventReader};

use crate::error::Result;
use crate::error::TrsError;

pub struct RssChannel {
    pub title: String,
    pub link: String,
    pub description: String,
    pub articles: Vec<RssArticle>,
}

pub struct RssArticle {
    pub title: String,
    pub link: String,
    pub description: String,
    pub date: Option<OffsetDateTime>,
}

impl RssChannel {
    fn new() -> Self {
        RssChannel {
            title: String::new(),
            link: String::new(),
            description: String::new(),
            articles: Vec::new(),
        }
    }

    fn update_channel_field(&mut self, field: &XmlTagField, value: String) -> Result<()> {
        let last_article = self.articles.last_mut();
        let no_item_error = || {
            TrsError::Error(format!(
                "No item found to update field <{}>",
                field.hierarchical_tag
            ))
        };

        match field.field {
            XmlField::ChannelTitle => self.title = value,
            XmlField::ChannelLink => self.link = value,
            XmlField::ChannelDescription => self.description = value,
            XmlField::ArticleTitle => last_article.ok_or_else(no_item_error)?.title = value,
            XmlField::ArticleLink => last_article.ok_or_else(no_item_error)?.link = value,
            XmlField::ArticleDescription => {
                last_article.ok_or_else(no_item_error)?.description = value
            }
            XmlField::ArticlePubDate => {
                last_article.ok_or_else(no_item_error)?.date = Some(RssArticle::parse_date(&value)?)
            }
        }

        Ok(())
    }
}

impl RssArticle {
    fn new() -> Self {
        RssArticle {
            title: String::new(),
            link: String::new(),
            description: String::new(),
            date: None,
        }
    }

    fn parse_date(value: &str) -> Result<OffsetDateTime> {
        let weird_format = format_description::parse(
            "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] UTC",
        )
        .unwrap();
        let parsed = OffsetDateTime::parse(value, &Rfc2822)
            .or(OffsetDateTime::parse(value, &Rfc3339))
            .or(OffsetDateTime::parse(value, &Iso8601::DEFAULT));

        match parsed {
            Ok(date) => Ok(date),
            Err(_) => {
                // Try parsing with the weird format
                PrimitiveDateTime::parse(value, &weird_format)
                    .map(|dt| dt.assume_utc())
                    .map_err(|e| {
                        TrsError::Error(format!("Failed to parse date '{}': {}", value, e))
                    })
            }
        }
    }
}

enum XmlField {
    ArticleTitle,
    ArticleLink,
    ArticlePubDate,
    ArticleDescription,
    ChannelTitle,
    ChannelLink,
    ChannelDescription,
}

struct XmlTagField {
    hierarchical_tag: &'static str,
    tag: &'static str,
    field: XmlField,
}

impl XmlTagField {
    const fn mapping(hierarchical_tag: &'static str, tag: &'static str, field: XmlField) -> Self {
        XmlTagField {
            hierarchical_tag,
            tag,
            field,
        }
    }

    fn corresponding_field(hierarchical_tag: &str) -> Option<&'static XmlTagField> {
        for field in FIELD_TAG_MAPPINGS.iter() {
            if field.hierarchical_tag == hierarchical_tag {
                return Some(field);
            }
        }

        None
    }
}

const FIELD_TAG_MAPPINGS: [XmlTagField; 7] = [
    XmlTagField::mapping("title", "title", XmlField::ChannelTitle),
    XmlTagField::mapping("link", "link", XmlField::ChannelLink),
    XmlTagField::mapping("description", "description", XmlField::ChannelDescription),
    XmlTagField::mapping("item > title", "title", XmlField::ArticleTitle),
    XmlTagField::mapping("item > link", "link", XmlField::ArticleLink),
    XmlTagField::mapping(
        "item > description",
        "description",
        XmlField::ArticleDescription,
    ),
    XmlTagField::mapping("item > pubDate", "pubDate", XmlField::ArticlePubDate),
];

pub fn parse_rss_channel<R: Read>(xml_source_stream: EventReader<R>) -> Result<RssChannel> {
    let mut channel = RssChannel::new();
    let mut tag_prefix = "";
    let mut current_field: Option<&XmlTagField> = None;
    for e in xml_source_stream {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => match name.local_name.as_str() {
                "item" => {
                    tag_prefix = "item > ";
                    channel.articles.push(RssArticle::new());
                }
                tag => {
                    let None = current_field else {
                        let current_field_name = current_field.unwrap();
                        return Err(TrsError::Error(format!(
                            "Unexpected <{}> start tag without closing existing tag <{}>",
                            tag, current_field_name.hierarchical_tag
                        )));
                    };

                    let tag_name_with_prefix = format!("{}{}", tag_prefix, tag);
                    current_field = XmlTagField::corresponding_field(&tag_name_with_prefix);
                }
            },
            Ok(XmlEvent::EndElement { name }) => match name.local_name.as_str() {
                "item" => {
                    let None = current_field else {
                        let current_field_name = current_field.unwrap();
                        return Err(TrsError::Error(format!(
                            "Unexpected </item> end tag without closing field {}",
                            current_field_name.hierarchical_tag
                        )));
                    };
                    tag_prefix = "";
                }
                tag => {
                    if let Some(field) = current_field.take() {
                        if field.tag == tag {
                            current_field = None;
                        } else {
                            return Err(TrsError::Error(format!(
                                "Unexpected </{}> end tag, expected </{}>",
                                tag, field.hierarchical_tag
                            )));
                        }
                    }
                }
            },
            Ok(XmlEvent::Characters(data)) => {
                if let Some(field) = current_field {
                    let err = channel.update_channel_field(field, data);
                    if let Err(e) = err {
                        eprintln!("Error updating channel field: {}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error parsing XML: {}", e);
                return Err(TrsError::XmlRsError(
                    e,
                    "Unexpected XML parsing error".to_string(),
                ));
            }
            _ => {}
        }
    }

    if channel.title.is_empty() || channel.link.is_empty() || channel.description.is_empty() {
        return Err(TrsError::Error("This is not a valid RSS feed".to_string()));
    }

    Ok(channel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xml::ParserConfig;

    macro_rules! validate_sample {
        ($test_name:ident, $file_name:literal, $title:literal, $link:literal, $description: literal, $article_count: literal) => {
            #[test]
            fn $test_name() {
                let bytes = include_bytes!(concat!("../sample/", $file_name));
                let xml_source_stream = ParserConfig::new()
                    .ignore_invalid_encoding_declarations(true)
                    .create_reader(&bytes[..]);
                let rss_channel = parse_rss_channel(xml_source_stream).unwrap();

                assert_eq!(rss_channel.title, $title);
                assert_eq!(rss_channel.link, $link);
                assert_eq!(rss_channel.description, $description);
                assert_eq!(rss_channel.articles.len(), $article_count);
                for article in &rss_channel.articles {
                    assert!(!article.title.is_empty());
                    assert!(!article.link.is_empty());
                    assert!(!article.description.is_empty());
                    assert!(article.date.is_some());
                }
            }
        };
    }

    validate_sample!(
        sample1,
        "rss.xml",
        "Bryce Vandegrift's Website",
        "https://brycev.com/",
        "Updates to Bryce Vandegrift's blog",
        28
    );

    validate_sample!(
        sample2,
        "rss2.xml",
        "ploeh blog",
        "https://blog.ploeh.dk",
        "danish software design",
        10
    );
}
