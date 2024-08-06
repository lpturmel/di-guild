use crate::config::CONFIG;
use crate::discord::api::{DiscordPayload, DiscordResponse, ResponseType};
use crate::discord::roles::Role;
use crate::error::{Error, Result};
use crate::AppState;
use di_core::RaidBots;
use std::str::FromStr;

pub async fn queue_sims(payload: &DiscordPayload, state: &AppState) -> Result<DiscordResponse> {
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

    let rb_client = RaidBots::new().set_cookie(&CONFIG.cookie).build();

    let res = (
        ResponseType::ChannelMessageWithSource,
        "Queueing sims...".to_string(),
    )
        .into();

    Ok(res)
}
