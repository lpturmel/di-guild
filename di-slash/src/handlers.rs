use crate::discord::api::{DiscordPayload, DiscordResponse, IntoDiscordResponse, ResponseType};
use crate::discord::commands::Commands;
use crate::error::Error;
use crate::{commands, AppState};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::str::FromStr;

pub async fn ping() -> impl IntoResponse {
    (ResponseType::Pong, "pong".to_string()).into_discord_response()
}

pub async fn webhook(
    State(state): State<AppState>,
    Json(payload): Json<DiscordPayload>,
) -> crate::error::Result<DiscordResponse> {
    match payload.payload_type {
        1 => Ok((ResponseType::Pong, "pong".to_string()).into_discord_response()),
        2 => {
            let int_data = &payload.data.as_ref().ok_or(Error::NoApplicationData)?;
            let command = Commands::from_str(&int_data.id)?;
            match command {
                Commands::QueueSims => Ok(commands::queue_sims(&payload, &state).await?),
                Commands::AddSimc => Ok(commands::add_simc(&payload, &state).await?),
            }
        }
        _ => Err(Error::BadDiscordRequest),
    }
}
