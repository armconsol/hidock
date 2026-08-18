use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid response format")]
    InvalidResponse,

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,
}

impl ApiError {
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            401 => Self::Unauthorized,
            404 => Self::NotFound,
            _ => Self::ApiError { status, message },
        }
    }
}
