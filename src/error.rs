use thiserror::Error;

#[derive(Error, Debug)]
pub enum TikTokError {
    #[error("TikTok returned invalid JSON response")]
    InvalidJSON(#[from] serde_json::Error),

    #[error("TikTok returned an empty response")]
    EmptyResponse,

    #[error("TikTok API error: {0}")]
    ApiError(String),

    #[error("TikTok captcha required")]
    CaptchaRequired,

    #[error("Object not found")]
    NotFound,

    #[error("Sound was removed by TikTok")]
    SoundRemoved,

    #[error("Browser automation error: {0}")]
    BrowserError(String),

    #[error("Failed to generate X-Bogus signature")]
    SignatureError,
    
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, TikTokError>;