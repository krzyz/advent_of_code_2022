#![feature(try_blocks)]
#![feature(get_many_mut)]

use std::{
    collections::HashSet,
    iter::{self, repeat},
    str::FromStr,
};

use anyhow::{anyhow, Result};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Move {
    Left(i32),
    Right(i32),
    Up(i32),
    Down(i32),
}

impl FromStr for Move {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let chars: [&str; 2] = s
            .split(char::is_whitespace)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|v: Vec<_>| anyhow!("Wrong number of elements: {}", v.len()))?;
        let l = chars[1].parse().map_err(anyhow::Error::msg)?;
        let mv = match chars[0] {
            "L" => Move::Left(l),
            "R" => Move::Right(l),
            "U" => Move::Up(l),
            "D" => Move::Down(l),
            _ => Err(anyhow!("Unrecognized direction: {}", chars[0]))?,
        };

        Ok(mv)
    }
}

impl Move {
    fn len(&self) -> i32 {
        match self {
            &Move::Left(l) => l,
            &Move::Right(l) => l,
            &Move::Up(l) => l,
            &Move::Down(l) => l,
        }
    }

    fn with(&self, l: i32) -> Move {
        match self {
            &Move::Left(_) => Move::Left(l),
            &Move::Right(_) => Move::Right(l),
            &Move::Up(_) => Move::Up(l),
            &Move::Down(_) => Move::Down(l),
        }
    }

    fn apply(&self, rope: &Vec<(i32, i32)>) -> Vec<(i32, i32)> {
        if self.len() == 1 {
            let mut rope_new = rope.clone();
            let head = rope_new.get_mut(0).unwrap();
            *head = match self {
                &Move::Left(l) => (head.0 - l, head.1),
                &Move::Right(l) => (head.0 + l, head.1),
                &Move::Up(l) => (head.0, head.1 + l),
                &Move::Down(l) => (head.0, head.1 - l),
            };

            for i in 0..rope.len()-1 {
                let [h, t] = rope_new.get_many_mut([i, i+1]).unwrap();

                let dist_x = h.0 - t.0;
                let dist_y = h.1 - t.1;

                if dist_x.abs() > 1 {
                    t.0 += dist_x.signum();
                    if dist_y.abs() == 1 {
                        t.1 += dist_y.signum();
                    }
                }
                if dist_y.abs() > 1 {
                    t.1 += dist_y.signum();
                    if dist_x.abs() == 1 {
                        t.0 += dist_x.signum();
                    }
                }
            }

            rope_new
        } else {
            panic!("Cannot apply more than one move")
        }
    }
}

pub fn count_unique_tail_positions(input: impl Iterator<Item = String>, rope_len: usize) -> Result<usize> {
    let t_pos = input
        .flat_map(|l| {
            let mv = l.parse::<Move>().unwrap();
            let len = mv.len();
            repeat(mv.with(1)).take(len as usize)
        })
        .scan(vec!((0, 0); rope_len), |rope, l| {
            *rope = l.apply(&*rope);
            Some(rope.clone())
        })
        .map(|rope| rope.last().copied().unwrap())
        .chain(iter::once((0, 0)))
        .collect::<HashSet<_>>();

    Ok(t_pos.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");
    const TEST_INPUT_2: &str = include_str!("../data/test_input_2");

    #[test]
    fn part1() {
        let res = count_unique_tail_positions(TEST_INPUT.lines().map(|l| l.to_string()), 2);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 13);
    }

    #[test]
    fn part2() {
        let res = count_unique_tail_positions(TEST_INPUT_2.lines().map(|l| l.to_string()), 10);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 36);
    }
}
