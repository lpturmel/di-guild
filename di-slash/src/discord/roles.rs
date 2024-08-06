use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    Raider,
    MythicDun,
    Officers,
    Dev,
    RaidLead,
}

impl Role {
    pub fn can_queue(&self) -> bool {
        match self {
            Role::Admin => true,
            Role::Officers => true,
            Role::Dev => true,
            Role::RaidLead => false,
            Role::Raider => false,
            Role::MythicDun => false,
        }
    }
}

impl FromStr for Role {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "1044039311638675568" => Ok(Role::Admin),
            "1050533864042418197" => Ok(Role::Raider),
            "1050534116220743772" => Ok(Role::MythicDun),
            "1050534316104495104" => Ok(Role::Officers),
            "1050585361362980884" => Ok(Role::Dev),
            "1258570247737446410" => Ok(Role::RaidLead),
            _ => Err(crate::error::Error::InvalidRole),
        }
    }
}
