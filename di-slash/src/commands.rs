use crate::discord::{DiscordResponse, ResponseType};
use crate::error::Result;

pub async fn queue_sims(_payload: &crate::discord::DiscordPayload) -> Result<DiscordResponse> {
    let res = (ResponseType::ChannelMessageWithSource, "Queued sims").into();
    Ok(res)
}
