use std::fmt::Display;

#[derive(Debug)]
pub enum TrsError {
    Error(String),
    XmlRsError(xml::reader::Error, String),
    SqlError(rusqlite::Error, String),
    ReqwestError(reqwest::Error, String),
}

impl From<rusqlite::Error> for TrsError {
    fn from(err: rusqlite::Error) -> Self {
        TrsError::SqlError(err, "No additional context provided".to_string())
    }
}

impl From<reqwest::Error> for TrsError {
    fn from(err: reqwest::Error) -> Self {
        TrsError::ReqwestError(err, "No additional context provided".to_string())
    }
}

impl Display for TrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TrsError::Error(msg) => format!("XML Parse Error: {}", msg),
                TrsError::XmlRsError(err, msg) => format!("{} XML Rs error {}", msg, err),
                TrsError::SqlError(err, msg) => format!("SQL Error: {} - {}", err, msg),
                TrsError::ReqwestError(err, msg) => format!("Reqwest Error: {} - {}", err, msg),
            }
        )
    }
}
