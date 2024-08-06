use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric1, char, line_ending, newline, not_line_ending},
    combinator::{opt, recognize},
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimcData {
    pub character_name: String,
    pub server: String,
    pub region: String,
    pub race: String,
    pub spec: String,
    pub class: String,
    pub level: u32,
    pub date: String,
    /// Represents the talent tree points
    pub talent_string: String,
}

fn parse_key_value_line(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(alphanumeric1, char('='), take_until("\n")),
        newline,
    )(input)
}

fn skip_comment_line(input: &str) -> IResult<&str, &str> {
    recognize(terminated(preceded(tag("#"), not_line_ending), newline))(input)
}
/// Parse the first line of the simc file
///
/// # Example
/// ```ignore
/// ## GhostMage - Frost - 2024-08-05 17:56 - US/Zul'jin
/// ```
///
/// returns
/// ```ignore
/// Ok(("", ("GhostMage", "Frost", "2024-08-05 17:56", "US", "Zul'jin")))
/// ```
fn parse_metadata(input: &str) -> IResult<&str, (&str, &str, &str, &str, &str)> {
    let (input, (character_name, spec, date, region, server, _)) = tuple((
        preceded(tag("# "), take_until(" - ")),
        preceded(tag(" - "), take_until(" - ")),
        preceded(tag(" - "), take_until(" - ")),
        preceded(tag(" - "), take_until("/")),
        preceded(tag("/"), take_until("\n")),
        newline,
    ))(input)?;

    Ok((input, (character_name, spec, date, region, server)))
}
/// Parse the 8 lines after the metadata
/// # Example
/// ```ignore
/// # Ghostmage - Frost - 2024-08-05 17:56 - US/Zul'jin
/// # SimC Addon 11.0.0-01
/// # WoW 11.0.0.55939, TOC 110000
/// # Requires SimulationCraft 1000-01 or newer
/// mage="Ghostmage"
/// level=70
/// race=night_elf
/// region=us
/// server=zuljin
/// role=spell
/// professions=alchemy=19/herbalism=26
/// spec=frost
/// talents=CAEArhxZfsv/vllYUrQS3iw2nPzYzsZBzwMziBzYmGjxYmZGGmBmZmZmZmZmZmZmZGzAAAAAAAAAAAAYWAAAAAAAAA
/// ```
///
/// returns
/// ```ignore
/// Ok(("", ("mage", "Ghostmage", "70", "night_elf", "us", "zuljin", "frost")))
/// ```
fn parse_character_info(input: &str) -> IResult<&str, (&str, &str, &str, &str, &str, &str, &str)> {
    let (input, (class, character_name)) = parse_key_value_line(input)?;
    let character_name = character_name
        .strip_prefix('\"')
        .map(|s| s.strip_suffix('\"').unwrap_or(character_name))
        .unwrap_or(character_name);
    let (input, (_, level)) = parse_key_value_line(input)?;
    let (input, (_, race)) = parse_key_value_line(input)?;
    let (input, (_, region)) = parse_key_value_line(input)?;
    let (input, (_, server)) = parse_key_value_line(input)?;
    // Role
    let (input, (_, _)) = parse_key_value_line(input)?;
    // Profession can be empty if the character has not learned any
    let (input, _) = opt(preceded(
        opt(line_ending),
        opt(preceded(
            tag("professions="),
            recognize(parse_key_value_line),
        )),
    ))(input)?;
    let (input, (_, spec)) = parse_key_value_line(input)?;
    Ok((
        input,
        (class, character_name, level, race, region, server, spec),
    ))
}
pub fn parse_simc(input: &str) -> IResult<&str, SimcData> {
    let (input, (m_character_name, m_spec, date, _, _)) = parse_metadata(input)?;
    // SimC Addon details
    let (input, _) = skip_comment_line(input)?;
    // Wow Version details
    let (input, _) = skip_comment_line(input)?;
    // SimC minimum version
    let (input, _) = skip_comment_line(input)?;

    // Line break between metadata and character info
    let (input, _) = newline(input)?;

    let (input, (class, character_name, level, race, region, server, spec)) =
        parse_character_info(input)?;
    assert_eq!(m_character_name, character_name);
    assert_eq!(m_spec.to_lowercase(), spec);

    // Line break between character info and talent tree
    let (input, _) = newline(input)?;

    let (input, (_, talents)) = parse_key_value_line(input)?;

    Ok((
        input,
        SimcData {
            character_name: character_name.to_string(),
            spec: spec.to_string(),
            date: date.to_string(),
            server: server.to_string(),
            region: region.to_string(),
            race: race.to_string(),
            class: class.to_string(),
            level: level.parse().expect("to parse level"),
            talent_string: talents.to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_value_line() {
        let input = "key=value\n";
        let res = parse_key_value_line(input);
        assert_eq!(res, Ok(("", ("key", "value"))));
    }

    #[test]
    fn test_parse_char_metadata() {
        let input = std::fs::read_to_string("tests/mage.txt").expect("to read file");
        let res = parse_metadata(&input);
        assert!(res.is_ok());
        let (_, (character_name, spec, date, region, server)) = res.unwrap();
        assert_eq!(character_name, "Ghostmage");
        assert_eq!(spec, "Frost");
        assert_eq!(date, "2024-08-05 17:56");
        assert_eq!(region, "US");
        assert_eq!(server, "Zul'jin");
    }
    #[test]
    fn test_parse_mage_character_info() {
        let input = std::fs::read_to_string("tests/mage.txt").expect("to read file");
        let res = parse_simc(&input);
        assert!(res.is_ok());
        let (_, simc) = res.unwrap();
        assert_eq!(simc.character_name, "Ghostmage");
        assert_eq!(simc.spec, "frost");
        assert_eq!(simc.date, "2024-08-05 17:56");
        assert_eq!(simc.server, "zuljin");
        assert_eq!(simc.region, "us");
        assert_eq!(simc.race, "night_elf");
        assert_eq!(simc.class, "mage");
        assert_eq!(simc.level, 70);
    }
    #[test]
    fn test_parse_warlock_character_info() {
        let input = std::fs::read_to_string("tests/warlock.txt").expect("to read file");
        let res = parse_simc(&input);
        assert!(res.is_ok());
        let (_, simc) = res.unwrap();
        assert_eq!(simc.character_name, "Locksuout");
        assert_eq!(simc.spec, "destruction");
        assert_eq!(simc.date, "2024-08-05 23:05");
        assert_eq!(simc.server, "zuljin");
        assert_eq!(simc.region, "us");
        assert_eq!(simc.race, "void_elf");
        assert_eq!(simc.class, "warlock");
        assert_eq!(simc.level, 70);
    }
    #[test]
    fn test_parse_paladin_character_info() {
        let input = std::fs::read_to_string("tests/paladin.txt").expect("to read file");
        let res = parse_simc(&input);
        assert!(res.is_ok());
        let (_, simc) = res.unwrap();
        assert_eq!(simc.character_name, "Doomdaim");
        assert_eq!(simc.spec, "holy");
        assert_eq!(simc.date, "2024-08-05 23:04");
        assert_eq!(simc.server, "zuljin");
        assert_eq!(simc.region, "us");
        assert_eq!(simc.race, "dwarf");
        assert_eq!(simc.class, "paladin");
        assert_eq!(simc.level, 70);
    }
    #[test]
    fn test_parse_dk_no_professions_character_info() {
        let input = std::fs::read_to_string("tests/dk.txt").expect("to read file");
        let res = parse_simc(&input);
        assert!(res.is_ok());
        let (_, simc) = res.unwrap();
        assert_eq!(simc.character_name, "Ghostdk");
        assert_eq!(simc.spec, "blood");
        assert_eq!(simc.date, "2024-08-05 23:31");
        assert_eq!(simc.server, "zuljin");
        assert_eq!(simc.region, "us");
        assert_eq!(simc.race, "dwarf");
        assert_eq!(simc.class, "deathknight");
        assert_eq!(simc.level, 70);
    }
}
