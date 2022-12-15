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
use rayon::prelude::*;
use util::{parse_nice, Span};

#[derive(Debug, Clone, Copy)]
struct Sensor {
    pos: (i64, i64),
    closest_beacon: (i64, i64),
    d: i64,
}

impl Sensor {
    fn new(pos: (i64, i64), closest_beacon: (i64, i64)) -> Self {
        Sensor {
            pos,
            closest_beacon,
            d: (pos.0 - closest_beacon.0).abs() + (pos.1 - closest_beacon.1).abs(),
        }
    }

    fn inside(&self, point: (i64, i64)) -> bool {
        ((point.0 - self.pos.0).abs() + (point.1 - self.pos.1).abs()) <= self.d
    }

    fn border(&self) -> Vec<(i64, i64)> {
        let top_right = (0..=(self.d + 1)).map(|i| (self.pos.0 + i, self.pos.1 + self.d + 1 - i));
        let bottom_right =
            (0..=(self.d + 1)).map(|i| (self.pos.0 + self.d + 1 - i, self.pos.1 - i));
        let bottom_left = (0..=(self.d + 1)).map(|i| (self.pos.0 - i, self.pos.1 - self.d - 1 + i));
        let top_left = (0..=(self.d + 1)).map(|i| (self.pos.0 - self.d - 1 + i, self.pos.1 + i));

        top_right
            .chain(bottom_right)
            .chain(bottom_left)
            .chain(top_left)
            .collect::<Vec<_>>()
    }
}

fn parse_number<'a, E>(i: Span<'a>) -> IResult<Span<'a>, i64, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map_res(recognize(tuple((opt(char('-')), digit1))), |i: Span<'a>| {
        FromStr::from_str(i.fragment())
    })(i)
}

fn parse_position<'a, E>(i: Span<'a>) -> IResult<Span<'a>, (i64, i64), E>
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
        |(pos, closest_beacon)| Sensor::new(pos, closest_beacon),
    )(i)
}

fn get_sensors(input: impl Iterator<Item = String>) -> Result<Vec<Sensor>> {
    input
        .map(|l| -> Result<Sensor> {
            parse_nice(l.as_str(), parse_sensor).ok_or(anyhow!("Couldn't parse line!"))
        })
        .collect::<Result<Vec<_>>>()
}

fn get_sensor_ranges(sensors: &[Sensor], y: i64) -> Vec<Option<(i64, i64)>> {
    sensors
        .iter()
        .map(
            |Sensor {
                 pos,
                 closest_beacon,
                 ..
             }| {
                let d = (pos.0 - closest_beacon.0).abs() + (pos.1 - closest_beacon.1).abs();
                let ly = (pos.1 - y).abs();
                let lx = d - ly;
                (lx > 0).then(|| (pos.0 - lx, pos.0 + lx))
            },
        )
        .collect::<Vec<_>>()
}

pub fn get_num_ruled_out(input: impl Iterator<Item = String>, y: i64) -> Result<usize> {
    let sensors = get_sensors(input)?;
    let ranges = get_sensor_ranges(sensors.as_slice(), y);
    let beacons_y = sensors
        .iter()
        .filter_map(|Sensor { closest_beacon, .. }| {
            (closest_beacon.1 == y).then(|| closest_beacon.0)
        })
        .collect::<HashSet<_>>();

    let line_in_range = ranges
        .iter()
        .filter_map(|&x| x)
        .flat_map(|(s, e)| (s..=e).collect::<Vec<_>>())
        .collect::<HashSet<_>>();

    let no_beacons_for_sure = line_in_range.difference(&beacons_y).collect::<Vec<_>>();

    Ok(no_beacons_for_sure.len())
}

pub fn get_distress_beacon_freq(
    input: impl Iterator<Item = String>,
    min: i64,
    max: i64,
) -> Result<i64> {
    let sensors = get_sensors(input)?;

    sensors
        .iter()
        .flat_map(|sensor| {
            sensor
                .border()
                .into_par_iter()
                .filter(|(x, y)| (min..=max).contains(x) && (min..=max).contains(y))
                .filter(|pos| sensors.iter().all(|sensor| !sensor.inside(*pos)))
                .collect::<Vec<_>>()
                .into_iter()
        })
        .next()
        .ok_or(anyhow!("Position of distress beacon not found!"))
        .map(|(x, y)| 4000000 * x + y)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_num_ruled_out(TEST_INPUT.lines().map(|l| l.to_string()), 10);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 26);
    }

    #[test]
    fn part2() {
        let res = get_distress_beacon_freq(TEST_INPUT.lines().map(|l| l.to_string()), 0, 20);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 56000011);
    }
}
