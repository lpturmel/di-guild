use crate::commands;
use crate::discord::Commands;
use crate::discord::DiscordResponse;
use crate::discord::IntoDiscordResponse;
use crate::discord::ResponseType;
use crate::error::Error;
use axum::response::IntoResponse;
use axum::Json;
use std::str::FromStr;

pub async fn ping() -> impl IntoResponse {
    (ResponseType::Pong, "pong".to_string()).into_discord_response()
}

pub async fn webhook(
    Json(payload): Json<crate::discord::DiscordPayload>,
) -> crate::error::Result<DiscordResponse> {
    match payload.payload_type {
        1 => Ok((ResponseType::Pong, "pong".to_string()).into_discord_response()),
        2 => {
            let int_data = &payload.data.as_ref().ok_or(Error::NoApplicationData)?;
            let command = Commands::from_str(&int_data.id)?;
            match command {
                Commands::QueueSims => Ok(commands::queue_sims(&payload).await?),
            }
        }
        _ => Err(Error::BadDiscordRequest),
    }
}
