use nom::bytes::complete::tag;
use nom::character::complete::alphanumeric1;
use nom::{
    bytes::complete::take_until,
    character::complete::{char, newline, not_line_ending},
    combinator::recognize,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SimcData {
    character_name: String,
    server: String,
    region: String,
    race: String,
    spec: String,
    class: String,
    level: u32,
    date: String,
    /// Represents the talent tree points
    talent_string: String,
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
    // Profession
    let (input, (_, _)) = parse_key_value_line(input)?;
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
    use std::time::Instant;

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
        println!("parse_metadata: {:?}", res);
        assert!(res.is_ok());
        let (_, (character_name, spec, date, region, server)) = res.unwrap();
        assert_eq!(character_name, "Ghostmage");
        assert_eq!(spec, "Frost");
        assert_eq!(date, "2024-08-05 17:56");
        assert_eq!(region, "US");
        assert_eq!(server, "Zul'jin");
    }
    #[test]
    fn test_parse_character_info() {
        let input = std::fs::read_to_string("tests/mage.txt").expect("to read file");
        let now = Instant::now();
        let res = parse_simc(&input);
        println!("parse_simc: {:?}", now.elapsed());
        assert!(res.is_ok());
        let (_, simc) = res.unwrap();
        println!("{:?}", simc);
        assert_eq!(simc.character_name, "Ghostmage");
        assert_eq!(simc.spec, "frost");
        assert_eq!(simc.date, "2024-08-05 17:56");
        assert_eq!(simc.server, "zuljin");
        assert_eq!(simc.region, "us");
        assert_eq!(simc.race, "night_elf");
        assert_eq!(simc.class, "mage");
        assert_eq!(simc.level, 70);
    }
}
