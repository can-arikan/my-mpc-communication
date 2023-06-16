use std::{ error::Error, fmt };

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommunicationError {
    description: String,
}

impl CommunicationError {
    pub fn new(description: String) -> Self {
        Self { description }
    }
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for CommunicationError {
    fn description(&self) -> &str {
        &self.description
    }
}
