use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    QueueSims,
    AddSimc,
}

impl FromStr for Commands {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "1050585792570986537" => Ok(Commands::QueueSims),
            "1270192536836903043" => Ok(Commands::AddSimc),
            _ => Err(crate::error::Error::InvalidCommand),
        }
    }
}
