#![feature(try_blocks)]
#![feature(iter_intersperse)]

use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    error::ParseError,
    multi::separated_list0,
    sequence::tuple,
    IResult,
};
use util::{parse_nice, Span};

pub enum Space {
    Air,
    Rock,
    Sand,
}

impl Space {
    fn is_filled(&self) -> bool {
        match self {
            Space::Air => false,
            Space::Rock | Space::Sand => true,
        }
    }
}

struct Cave {
    space: HashMap<(i32, i32), Space>,
    moving: Option<(i32, i32)>,
    bottom: i32,
}

pub enum Progress {
    Falling,
    EndFall,
    FallVoid,
}

impl Cave {
    fn from_lines(lines: &[Line]) -> Self {
        let mut space = HashMap::new();
        let mut bottom = None;

        for line in lines {
            for window in line.points.windows(2) {
                let (x1, y1) = window[0];
                let (x2, y2) = window[1];

                if let Some(bottom) = bottom {
                    bottom = y1.min(bottom);
                    bottom = y2.min(bottom);
                }

                if x1 == x2 {
                    for y in y1..=y2 {
                        *space.get_mut(&(x1, y)).unwrap() = Space::Rock;
                    }
                } else if y1 == y2 {
                    for x in x1..=x2 {
                        *space.get_mut(&(x, y1)).unwrap() = Space::Rock;
                    }
                }
            }
        }

        Self {
            space,
            moving: None,
            bottom: bottom.unwrap(),
        }
    }
    fn add_sand(&mut self, loc: &(i32, i32)) -> Option<()> {
        let space = self.space.get_mut(loc)?;
        match &space {
            &Space::Air => {
                *space = Space::Sand;
                self.moving = Some(*loc);
                Some(())
            }
            _ => None,
        }
    }

    fn progress(&mut self) -> Progress {
        let falling = if let Some((x, y)) = &self.moving {
            let space_down = self.space.get_mut(&(*x, y - 1)).unwrap();
            let moved = if space_down.is_filled() {
                if !self.space[&(x - 1, y - 1)].is_filled() {
                    *self.space.get_mut(&(x - 1, y - 1)).unwrap() = Space::Rock;
                    true
                } else if !self.space[&(x + 1, y - 1)].is_filled() {
                    *self.space.get_mut(&(x + 1, y - 1)).unwrap() = Space::Rock;
                    true
                } else {
                    false
                }
            } else {
                *space_down = Space::Rock;
                true
            };

            if moved {
                *self.space.get_mut(&(*x, *y)).unwrap() = Space::Air;
            }

            moved
        } else {
            false
        }

        if 
    }
}

#[derive(Debug, Clone)]
struct Line {
    points: Vec<(i32, i32)>,
}

fn parse_number<'a, E>(i: Span<'a>) -> IResult<Span<'a>, i32, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map_res(digit1, |i: Span<'a>| FromStr::from_str(i.fragment()))(i)
}

fn parse_line<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Line, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map(
        separated_list0(
            tag(" -> "),
            map(
                tuple((parse_number, tag(","), parse_number)),
                |(x, _, y)| (x, y),
            ),
        ),
        |l| Line { points: l },
    )(i)
}

fn get_lines(input: impl Iterator<Item = String>) -> Result<impl Iterator<Item = Line>> {
    Ok(input
        .map(|l| -> Result<Line> {
            parse_nice(l.as_str(), parse_line).ok_or(anyhow!("Couldn't parse line!"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter())
}

pub fn get_num_sand_rest(input: impl Iterator<Item = String>) -> Result<usize> {
    let lines = get_lines(input)?.collect::<Vec<_>>();
    let cave = Cave::from_lines(lines.as_slice());
    let mut 
    println!("{x:#?}");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_num_sand_rest(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 24);
    }
}
