use std::fmt::Display;

use xml::reader::Error;

#[derive(Debug)]
pub enum TrsError {
    XmlParseError(String),
    XmlRsError(Error),
}

impl Display for TrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TrsError::XmlParseError(msg) => format!("XML Parse Error: {}", msg),
                TrsError::XmlRsError(err) => format!("XML Reader Error: {}", err),
            }
        )
    }
}
