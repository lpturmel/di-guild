use std::str::FromStr;

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
