use crate::discord::api::{InteractionResponse, ResponseType};
use axum::response::IntoResponse;
use ed25519_dalek::SignatureError;
use hex::FromHexError;
use reqwest::StatusCode;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingSignature,
    MissingTimestamp,
    BadSignature,
    InvalidCommand,
    InvalidBody,
    InvalidRole,
    BadDiscordRequest,
    NoApplicationData,
    MissingRole,
    BadDiscordAttachment,
    InvalidSimcString,
    Sql(libsql::Error),
    Json(serde_json::Error),
    SqsError(String),
    NoSimsToQueue,
}
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, String) {
        match self {
            Error::MissingSignature => (StatusCode::BAD_REQUEST, "Missing signature".to_string()),
            Error::MissingTimestamp => (StatusCode::BAD_REQUEST, "Missing timestamp".to_string()),
            Error::BadDiscordRequest => (StatusCode::BAD_REQUEST, "Bad request type".to_string()),
            Error::InvalidRole => (StatusCode::BAD_REQUEST, "Invalid role".to_string()),
            Error::BadSignature => (StatusCode::UNAUTHORIZED, "Bad signature".to_string()),
            Error::Json(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Error::InvalidBody => (StatusCode::BAD_REQUEST, "Invalid body".to_string()),
            Error::NoApplicationData => {
                (StatusCode::BAD_REQUEST, "No application data".to_string())
            }
            Error::InvalidCommand => (
                StatusCode::BAD_REQUEST,
                "Invalid command, not recognized".to_string(),
            ),

            Error::MissingRole => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    "Not authorized to queue sims".to_string(),
                ))
                .unwrap(),
            ),
            Error::BadDiscordAttachment => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    "The attachment is either missing or expired".to_string(),
                ))
                .unwrap(),
            ),
            Error::Sql(e) => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    format!("Sql error: {:?}", e),
                ))
                .unwrap(),
            ),
            Error::InvalidSimcString => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    "Invalid simc string".to_string(),
                ))
                .unwrap(),
            ),
            Error::NoSimsToQueue => (
                StatusCode::OK,
                serde_json::to_string(&InteractionResponse::new(
                    ResponseType::ChannelMessageWithSource,
                    "No sim strings found, skipping SQS batch".to_string(),
                ))
                .unwrap(),
            ),
            Error::SqsError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong with SQS".to_string(),
            ),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, error) = self.client_status_and_error();
        tracing::error!("Error: {:?}", self);
        (status, error).into_response()
        // response.extensions_mut().insert(self);
    }
}

impl From<FromHexError> for Error {
    fn from(_: FromHexError) -> Self {
        Error::BadSignature
    }
}

impl From<SignatureError> for Error {
    fn from(_: SignatureError) -> Self {
        Error::BadSignature
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Error::Json(e)
    }
}

impl From<libsql::Error> for Error {
    fn from(e: libsql::Error) -> Self {
        Error::Sql(e)
    }
}
