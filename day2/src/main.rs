#![feature(try_blocks)]

use anyhow::{anyhow, Result};
use std::io::{self, BufRead};
use thiserror::Error;

#[derive(Debug)]
enum MatchResult {
    Lose,
    Draw,
    Win,
}

impl MatchResult {
    fn value(self: &Self) -> i32 {
        match self {
            Self::Lose => 0,
            Self::Draw => 3,
            Self::Win => 6,
        }
    }
}

#[derive(Error, Debug)]
#[error("Error parsing into MatchResult")]
struct MatchResultParseError;

impl TryFrom<&str> for MatchResult {
    type Error = MatchResultParseError;

    fn try_from(shape: &str) -> Result<Self, Self::Error> {
        if shape.len() != 1 {
            return Err(MatchResultParseError);
        }

        match shape {
            "X" => Ok(Self::Lose),
            "Y" => Ok(Self::Draw),
            "Z" => Ok(Self::Win),
            _ => Err(MatchResultParseError),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

#[derive(Error, Debug)]
#[error("Error parsing into Shape")]
struct ShapeParseError;

impl TryFrom<&str> for Shape {
    type Error = ShapeParseError;

    fn try_from(shape: &str) -> Result<Self, Self::Error> {
        if shape.len() != 1 {
            return Err(ShapeParseError);
        }

        match shape {
            "A" | "X" => Ok(Self::Rock),
            "B" | "Y" => Ok(Self::Paper),
            "C" | "Z" => Ok(Self::Scissors),
            _ => Err(ShapeParseError),
        }
    }
}

impl Shape {
    fn value(self: &Self) -> i32 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }

    fn match_result(shape1: &Self, shape2: &Self) -> MatchResult {
        match (shape1, shape2) {
            (Self::Rock, Self::Paper)
            | (Self::Paper, Self::Scissors)
            | (Self::Scissors, Self::Rock) => MatchResult::Win,
            (s1, s2) if s1 == s2 => MatchResult::Draw,
            _ => MatchResult::Lose,
        }
    }

    fn from_result(shape1: &Self, result: &MatchResult) -> Self {
        match (shape1, result) {
            (s, MatchResult::Draw) => *s,
            (Self::Rock, MatchResult::Lose) | (Self::Paper, MatchResult::Win) => Self::Scissors,
            (Self::Paper, MatchResult::Lose) | (Self::Scissors, MatchResult::Win) => Self::Rock,
            (Self::Scissors, MatchResult::Lose) | (Self::Rock, MatchResult::Win) => Self::Paper,
        }
    }

    fn match_value(shape1: &Shape, shape2: &Shape) -> i32 {
        Self::match_result(shape1, shape2).value() + shape2.value()
    }
}

fn _find_total(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    input
        .map(|line| try {
            let line: String = line.into();
            let shapes: [Shape; 2] = line
                .split(' ')
                .map(|s| s.try_into())
                .collect::<Result<Vec<Shape>, _>>()?
                .try_into()
                .map_err(|e: Vec<_>| anyhow!("wrong number of shapes: {}", e.len()))?;

            Shape::match_value(&shapes[0], &shapes[1])
        })
        .sum()
}

fn find_total_2(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    input
        .map(|line| try {
            let line: String = line.into();
            let chars: Vec<&str> = line.split(' ').collect();
            let (shape, match_result): (Shape, MatchResult) = match (chars.get(0), chars.get(1)) {
                (Some(s), Some(mr)) => Ok(((*s).try_into()?, (*mr).try_into()?)),
                _ => Err(anyhow!("wrong number of chars in line: {}", chars.len())),
            }?;

            let shape2 = Shape::from_result(&shape, &match_result);
            Shape::match_value(&shape, &shape2)
        })
        .sum()
}

fn main() {
    let stdin = io::stdin();

    let total = find_total_2(stdin.lock().lines().filter_map(|s| s.ok())).unwrap();

    println!("{total}");
}
