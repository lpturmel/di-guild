use crate::config::CONFIG;
use crate::discord::api::{DiscordPayload, DiscordResponse, ResponseType};
use crate::discord::roles::Role;
use crate::error::{Error, Result};
use crate::AppState;
use di_core::RaidBots;
use std::str::FromStr;

pub async fn queue_sims(payload: &DiscordPayload, _state: &AppState) -> Result<DiscordResponse> {
    let authorized = payload
        .member
        .as_ref()
        .and_then(|m| {
            m.roles.iter().find_map(|r| {
                Role::from_str(r)
                    .ok()
                    .and_then(|role| if role.can_queue() { Some(true) } else { None })
            })
        })
        .unwrap_or(false);
    if !authorized {
        return Err(Error::MissingRole);
    }

    let _ = RaidBots::new().set_cookie(&CONFIG.cookie).build();

    let res = (
        ResponseType::ChannelMessageWithSource,
        "Queueing sims...".to_string(),
    )
        .into();

    Ok(res)
}

pub async fn add_simc(payload: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
    // NOTE: Safe to unwrap because the slash command options are required and validated by discord
    let attachments = payload
        .data
        .as_ref()
        .expect("to have data")
        .resolved
        .as_ref()
        .expect("to have resolved")
        .attachments
        .as_ref()
        .expect("to have attachments");
    let user_id = payload
        .member
        .as_ref()
        .expect("to have member")
        .user
        .id
        .as_str();
    let file_url = &attachments.values().next().expect("to have file").url;
    let res = reqwest::get(file_url)
        .await
        .map_err(|_| Error::BadDiscordAttachment)?;
    let attachment_text = res.text().await.map_err(|_| Error::BadDiscordAttachment)?;

    let (_, simc_data) =
        di_core::simc::parse_simc(&attachment_text).map_err(|_| Error::InvalidSimcString)?;
    let current_datetime = chrono::Utc::now();
    let formatted_datetime = current_datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let character_name = simc_data.character_name.to_lowercase();

    let _ = state
        .db
        .execute(
            "INSERT INTO sim_details (user_id, name, sim_str, added_at) VALUES (?,?,?,?)",
            libsql::params![
                user_id,
                character_name.as_str(),
                attachment_text,
                formatted_datetime
            ],
        )
        .await?;

    Ok((
        ResponseType::ChannelMessageWithSource,
        "Successfully added simc".to_string(),
    )
        .into())
}
