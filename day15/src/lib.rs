#![feature(try_blocks)]
#![feature(iter_intersperse)]

use std::collections::HashSet;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use nom::combinator::{opt, recognize};
use nom::sequence::preceded;
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, map_res},
    error::ParseError,
    sequence::tuple,
    IResult,
};
use util::{parse_nice, Span};

#[derive(Debug, Clone, Copy)]
struct Sensor {
    pos: (i32, i32),
    closest_beacon: (i32, i32),
}

fn parse_number<'a, E>(i: Span<'a>) -> IResult<Span<'a>, i32, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map_res(recognize(tuple((opt(char('-')), digit1))), |i: Span<'a>| {
        FromStr::from_str(i.fragment())
    })(i)
}

fn parse_position<'a, E>(i: Span<'a>) -> IResult<Span<'a>, (i32, i32), E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    tuple((
        preceded(tag("x="), parse_number),
        preceded(tag(", y="), parse_number),
    ))(i)
}

fn parse_sensor<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Sensor, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map(
        tuple((
            preceded(tag("Sensor at "), parse_position),
            preceded(tag(": closest beacon is at "), parse_position),
        )),
        |(pos, closest_beacon)| Sensor {
            pos,
            closest_beacon,
        },
    )(i)
}

fn get_sensors(input: impl Iterator<Item = String>) -> Result<Vec<Sensor>> {
    input
        .map(|l| -> Result<Sensor> {
            parse_nice(l.as_str(), parse_sensor).ok_or(anyhow!("Couldn't parse line!"))
        })
        .collect::<Result<Vec<_>>>()
}

fn get_sensor_ranges(sensors: &[Sensor], y: i32) -> Vec<Option<(i32, i32)>> {
    sensors
        .iter()
        .map(
            |Sensor {
                 pos,
                 closest_beacon,
             }| {
                let d = (pos.0 - closest_beacon.0).abs() + (pos.1 - closest_beacon.1).abs();
                let ly = (pos.1 - y).abs();
                let lx = d - ly;
                (lx > 0).then(|| (pos.0 - lx, pos.0 + lx))
            },
        )
        .collect::<Vec<_>>()
}

pub fn get_num_ruled_out(input: impl Iterator<Item = String>) -> Result<usize> {
    let sensors = get_sensors(input)?;
    let ranges = get_sensor_ranges(sensors.as_slice(), 10);
    let sensors_y = sensors
        .iter()
        .filter_map(|Sensor { closest_beacon, .. }| {
            (closest_beacon.1 == 10).then(|| closest_beacon.0)
        })
        .collect::<HashSet<_>>();

    Ok(ranges
        .iter()
        .filter_map(|&x| x)
        .flat_map(|(s, e)| (s..=e).collect::<Vec<_>>())
        .filter(|x| !sensors_y.contains(x))
        .collect::<HashSet<_>>()
        .len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_num_ruled_out(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 26);
    }
}
