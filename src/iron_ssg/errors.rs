use handlebars;
use json;
use serde_json::Error as JsonError;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use tera::Error as TeraError;

#[derive(Debug)]
pub enum IronSSGError {
    TeraError(TeraError),
    JsonError(JsonError),
    InvalidJSON(json::Error),
    FileError(io::Error),
    RenderError(handlebars::RenderError),
    CustomError(String),
    GenericError(String),
}

impl fmt::Display for IronSSGError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IronSSGError::InvalidJSON(err) => write!(f, "Invalid JSON: {}", err),
            IronSSGError::FileError(err) => write!(f, "File error: {}", err),
            IronSSGError::RenderError(err) => write!(f, "Rendering error: {}", err),
            IronSSGError::CustomError(err) => write!(f, "{}", err),
            IronSSGError::GenericError(err) => write!(f, "{}", err),
            IronSSGError::TeraError(err) => write!(f, "Tera error: {}", err),
            IronSSGError::JsonError(err) => write!(f, "JSON error: {}", err),
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

// Implement conversion from Box<dyn Error> to IronSSGError
impl From<Box<dyn StdError>> for IronSSGError {
    fn from(error: Box<dyn StdError>) -> Self {
        IronSSGError::GenericError(error.to_string())
    }
}

impl From<TeraError> for IronSSGError {
    fn from(err: TeraError) -> IronSSGError {
        IronSSGError::TeraError(err)
    }
}

impl From<JsonError> for IronSSGError {
    fn from(err: JsonError) -> IronSSGError {
        IronSSGError::JsonError(err)
    }
}
