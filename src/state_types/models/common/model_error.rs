use crate::state_types::EnvError;
use crate::state_types::RequestBuilderError;
use crate::types::api::APIErr;
use serde::Serialize;
use std::error::Error;
use std::fmt;

// TODO find a better name for this
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ModelError {
    API { message: String, code: u64 },
    Env { message: String },
    RequestBuilder { message: String },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ModelError::API { message, code } => write!(f, "{} {}", message, code),
            ModelError::Env { message } | ModelError::RequestBuilder { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl Error for ModelError {
    fn description(&self) -> &str {
        match &self {
            ModelError::API { message, .. }
            | ModelError::Env { message }
            | ModelError::RequestBuilder { message } => message,
        }
    }
}

impl From<APIErr> for ModelError {
    fn from(error: APIErr) -> Self {
        ModelError::API {
            message: error.message.to_owned(),
            code: error.code.to_owned(),
        }
    }
}

impl From<EnvError> for ModelError {
    fn from(error: EnvError) -> Self {
        ModelError::Env {
            message: error.to_string(),
        }
    }
}

impl From<RequestBuilderError> for ModelError {
    fn from(error: RequestBuilderError) -> Self {
        ModelError::RequestBuilder {
            message: error.to_string(),
        }
    }
}
