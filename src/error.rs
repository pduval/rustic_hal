use std::error::Error;
use std::fmt;
use serde_json::{Error as JsonError};

#[derive(Debug)]
pub enum HalError {
    Json(JsonError),
    Custom(String)
}

pub type HalResult<T> = Result<T, HalError>;

impl fmt::Display for HalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HalError::Json(ref e) => write!(f, "JSON Error: {}", e),
            HalError::Custom(ref s) => write!(f, "Notify error: {}", s)
        }
    }
}

impl Error for HalError {
    fn description(&self) -> &str {
        match *self {
            HalError::Json(_) => "Error in json processing",
            HalError::Custom(_) => "Internal Hal Error"
        }
    }
}

impl From<JsonError> for HalError {
    fn from(error: JsonError) -> Self {
        HalError::Json(error)
    }
}

