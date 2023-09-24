use handlebars;
use json; // Make sure the json crate is in your Cargo.toml
use std::error::Error as StdError;
use std::fmt;
use std::io; // Likewise for the handlebars crate

#[derive(Debug)]
pub enum IronSSGError {
    InvalidJSON(json::Error),
    FileError(io::Error),
    RenderError(handlebars::RenderError),
}

impl fmt::Display for IronSSGError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IronSSGError::InvalidJSON(err) => write!(f, "Invalid JSON: {}", err),
            IronSSGError::FileError(err) => write!(f, "File error: {}", err),
            IronSSGError::RenderError(err) => write!(f, "Rendering error: {}", err),
        }
    }
}

impl StdError for IronSSGError {}

impl From<handlebars::RenderError> for IronSSGError {
    fn from(err: handlebars::RenderError) -> IronSSGError {
        IronSSGError::RenderError(err)
    }
}

impl From<io::Error> for IronSSGError {
    fn from(err: io::Error) -> IronSSGError {
        IronSSGError::FileError(err)
    }
}

impl From<json::Error> for IronSSGError {
    fn from(err: json::Error) -> IronSSGError {
        IronSSGError::InvalidJSON(err)
    }
}
