#![allow(clippy::new_ret_no_self)]
use crate::error::Result;
use axum::response::IntoResponse;
use axum::Json;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex::FromHex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    QueueSims,
}

impl FromStr for Commands {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "1050585792570986537" => Ok(Commands::QueueSims),
            _ => Err(crate::error::Error::InvalidCommand),
        }
    }
}
/// Verify an ed25519 signature
/// used for validating discord webhooks
pub fn verify_sig(
    body: &str,
    signature: &str,
    timestamp: &str,
    public_key: String,
) -> Result<bool> {
    let public_key_bytes = <[u8; 32]>::from_hex(public_key)?;
    let public_key = VerifyingKey::from_bytes(&public_key_bytes)?;

    // Decode the signature from hex
    let signature_bytes = <[u8; 64]>::from_hex(signature)?;
    let signature = Signature::from_bytes(&signature_bytes);
    let timestamp_data = timestamp.as_bytes();
    let body_data = body.as_bytes();
    let message = [timestamp_data, body_data].concat();
    let res = public_key.verify(&message, &signature);
    Ok(res.is_ok())
}

pub enum ResponseType {
    Pong,
    ChannelMessageWithSource,
    DeferredChannelMessageWithSource,
    DeferredUpdateMessage,
    UpdateMessage,
    ApplicationCommandAutocompleteResult,
    Modal,
}

impl ResponseType {
    fn to_int(&self) -> u64 {
        match self {
            ResponseType::Pong => 1,
            ResponseType::ChannelMessageWithSource => 4,
            ResponseType::DeferredChannelMessageWithSource => 5,
            ResponseType::DeferredUpdateMessage => 6,
            ResponseType::UpdateMessage => 7,
            ResponseType::ApplicationCommandAutocompleteResult => 8,
            ResponseType::Modal => 9,
        }
    }
}
#[derive(Debug)]
pub struct InteractionResponse;

impl InteractionResponse {
    pub fn new<S: Into<String>>(r#type: ResponseType, content: S) -> DiscordResponse {
        DiscordResponse {
            r#type: r#type.to_int(),
            data: DiscordResponseData {
                content: content.into(),
                flags: 0,
                tts: false,
                embeds: None,
            },
        }
    }
}

pub trait IntoDiscordResponse {
    fn into_discord_response(self) -> DiscordResponse;
}

impl IntoDiscordResponse for (ResponseType, String) {
    fn into_discord_response(self) -> DiscordResponse {
        InteractionResponse::new(self.0, self.1)
    }
}

impl IntoResponse for DiscordResponse {
    fn into_response(self) -> axum::response::Response {
        let res = (StatusCode::OK, Json(self));
        info!("IntoResponse: {:?}", res);
        res.into_response()
    }
}

impl From<(ResponseType, String)> for DiscordResponse {
    fn from(t: (ResponseType, String)) -> Self {
        InteractionResponse::new(t.0, t.1)
    }
}

impl From<(ResponseType, &str)> for DiscordResponse {
    fn from(t: (ResponseType, &str)) -> Self {
        InteractionResponse::new(t.0, t.1.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordOption {
    pub name: String,
    pub r#type: u8,
    pub value: Option<serde_json::Value>,
    pub options: Option<Vec<DiscordOption>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordData {
    pub id: String,
    pub name: String,
    pub r#type: u64,
    pub options: Option<Vec<DiscordOption>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUser {
    pub avatar: Option<String>,
    pub avatar_decoration: Option<String>,
    pub discriminator: String,
    pub id: String,
    pub public_flags: u64,
    pub username: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordMember {
    pub roles: Vec<String>,
    pub user: DiscordUser,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordPayload {
    pub application_id: String,
    pub channel_id: Option<String>,
    /// The data of the incoming integration command
    pub data: Option<DiscordData>,
    pub guild_id: Option<String>,
    pub member: Option<DiscordMember>,
    #[serde(rename = "type")]
    pub payload_type: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordResponseData {
    content: String,
    flags: u64,
    tts: bool,
    embeds: Option<Vec<serde_json::Value>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordResponse {
    r#type: u64,
    data: DiscordResponseData,
}
