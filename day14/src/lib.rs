#![feature(try_blocks)]
#![feature(iter_intersperse)]

use std::{collections::HashMap, fmt, str::FromStr};
use std::{thread, time};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
struct Cave {
    contents: HashMap<(i32, i32), Space>,
    moving: Option<(i32, i32)>,
    bottom: i32,
}

impl fmt::Display for Cave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x_start, x_end, y_start, y_end) = self
            .contents
            .keys()
            .fold(None, |range: Option<(i32, i32, i32, i32)>, &(x, y)| {
                if let Some((x_start, x_end, y_start, y_end)) = range {
                    Some((x_start.min(x), x_end.max(x), y_start.min(y), y_end.max(y)))
                } else {
                    Some((x, x, y, y))
                }
            })
            .unwrap();

        for y in y_start..=y_end {
            for x in x_start..=x_end {
                let c = match self.get(&(x, y)) {
                    Space::Air => "  ",
                    Space::Rock => "██",
                    Space::Sand => "::",
                };

                write!(f, "{c}")?
            }
            write!(f, "\n")?
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    Falling,
    EndFall,
    FallVoid,
}

impl Cave {
    fn get(&self, loc: &(i32, i32)) -> Space {
        self.contents.get(loc).copied().unwrap_or(Space::Air)
    }

    fn from_lines(lines: &[Line]) -> Self {
        let mut contents = HashMap::new();
        let mut bottom = None;

        for line in lines {
            for window in line.points.windows(2) {
                let (x1, y1) = window[0];
                let (x2, y2) = window[1];

                if let Some(ref mut bottom) = bottom {
                    *bottom = y1.max(*bottom);
                    *bottom = y2.max(*bottom);
                } else {
                    bottom = Some(y1.max(y2))
                }

                if x1 == x2 {
                    for y in (y1..=y2).chain(y2..=y1) {
                        contents.insert((x1, y), Space::Rock);
                    }
                } else if y1 == y2 {
                    for x in (x1..=x2).chain(x2..=x1) {
                        contents.insert((x, y1), Space::Rock);
                    }
                }
            }
        }

        Self {
            contents,
            moving: None,
            bottom: bottom.unwrap(),
        }
    }

    fn add_sand(&mut self, loc: &(i32, i32)) -> Option<()> {
        let space = self.get(loc);
        match space {
            Space::Air => {
                self.contents.insert(*(loc), Space::Sand);
                self.moving = Some(*loc);
                Some(())
            }
            _ => None,
        }
    }

    fn progress(&mut self, floor: bool) -> Progress {
        let falling = if let Some((x, y)) = &self.moving {
            let down = (*x, y + 1);
            let move_to = if self.get(&down).is_filled() {
                let down_left = (x - 1, y + 1);
                let down_right = (x + 1, y + 1);
                if !self.get(&down_left).is_filled() {
                    Some(down_left)
                } else if !self.get(&down_right).is_filled() {
                    Some(down_right)
                } else {
                    None
                }
            } else {
                Some(down)
            };

            if let Some(move_to) = move_to {
                self.contents.remove(&(*x, *y));
                self.contents.insert(move_to, Space::Sand);
                self.moving = Some(move_to)
            }

            move_to
        } else {
            None
        };

        if let Some(falled_to) = falling {
            if falled_to.1 >= self.bottom {
                if floor {
                    if falled_to.1 >= self.bottom + 1 {
                        Progress::EndFall
                    } else {
                        Progress::Falling
                    }
                } else {
                    Progress::FallVoid
                }
            } else {
                Progress::Falling
            }
        } else {
            self.moving = None;
            Progress::EndFall
        }
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

pub fn get_num_sand_rest(
    input: impl Iterator<Item = String>,
    floor: bool,
    print: bool,
) -> Result<usize> {
    let lines = get_lines(input)?.collect::<Vec<_>>();
    let mut cave = Cave::from_lines(lines.as_slice());
    let mut progress = Progress::Falling;
    let mut rested = 0;
    let mut sand_ok = cave.add_sand(&(500, 0));
    while progress != Progress::FallVoid && sand_ok.is_some() {
        if print {
            let x = format!("{cave}");
            println!(
                "{}",
                x.lines()
                    .map(|l| l.chars().skip(0).take(160).collect::<String>())
                    .take(40)
                    .intersperse("\n".to_string())
                    .collect::<String>()
            );
            let dt = time::Duration::from_millis(50);

            thread::sleep(dt);
            std::process::Command::new("clear").status().unwrap();
        }

        match progress {
            Progress::Falling => {
                progress = cave.progress(floor);
            }
            Progress::EndFall => {
                rested += 1;
                sand_ok = cave.add_sand(&(500, 0));
                progress = Progress::Falling;
            }
            _ => (),
        }
    }
    Ok(rested)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_num_sand_rest(TEST_INPUT.lines().map(|l| l.to_string()), false, false);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 24);
    }

    #[test]
    fn part2() {
        let res = get_num_sand_rest(TEST_INPUT.lines().map(|l| l.to_string()), true, false);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 93);
    }
}
