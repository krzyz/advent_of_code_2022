#![feature(try_blocks)]
#![feature(iter_array_chunks)]
#![feature(get_many_mut)]

use anyhow::Result;
use std::{collections::VecDeque, str::FromStr};
use thiserror::Error;

#[derive(Debug)]
pub struct Move {
    n: i32,
    from: usize,
    to: usize,
}

#[derive(Debug)]
pub struct Stacks {
    stacks: Vec<VecDeque<char>>,
}

#[derive(Error, Debug)]
#[error("Unable to move")]
pub struct MoveError;

impl Stacks {
    fn apply_move(&mut self, mv: Move) -> Result<(), MoveError> {
        if let Ok([stack_from, stack_to]) = self.stacks.get_many_mut([mv.from - 1, mv.to - 1]) {
            for _ in 0..mv.n {
                stack_to.push_front(stack_from.pop_front().ok_or(MoveError)?);
            }
        }

        Ok(())
    }

    fn apply_move_crate_mover9001(&mut self, mv: Move) -> Result<(), MoveError> {
        if let Ok([stack_from, stack_to]) = self.stacks.get_many_mut([mv.from - 1, mv.to - 1]) {
            let mut tmp = VecDeque::new();

            for _ in 0..mv.n {
                tmp.push_front(stack_from.pop_front().ok_or(MoveError)?);
            }
            for _ in 0..mv.n {
                stack_to.push_front(tmp.pop_front().ok_or(MoveError)?);
            }
        }

        Ok(())
    }

    fn top_crates(&self) -> String {
        self.stacks
            .iter()
            .map(|stack| stack.front().unwrap_or(&' '))
            .collect()
    }
}

#[derive(Error, Debug)]
pub enum MoveParseError {
    #[error("Int parse error parsing into Move")]
    ParseNumberError(#[from] std::num::ParseIntError),

    #[error("Usize parse error parsing into Move")]
    CastToUsizeError(#[from] std::num::TryFromIntError),

    #[error("Wrong number of numbers when parsing into Move")]
    WrongNumberError(usize),
}

#[derive(Error, Debug)]
pub enum StacksParseError {
    #[error("Missing stacks description")]
    MissingStacks,
}

impl FromStr for Move {
    type Err = MoveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nums: [i32; 3] = s
            .split(' ')
            .skip(1)
            .step_by(2)
            .map(|n| n.trim().parse().map_err(MoveParseError::ParseNumberError))
            .collect::<Result<Vec<i32>, _>>()?
            .try_into()
            .map_err(|v: Vec<_>| MoveParseError::WrongNumberError(v.len()))?;

        Ok(Self {
            n: nums[0],
            from: usize::try_from(nums[1]).map_err(MoveParseError::CastToUsizeError)?,
            to: usize::try_from(nums[2]).map_err(MoveParseError::CastToUsizeError)?,
        })
    }
}

impl FromStr for Stacks {
    type Err = StacksParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals_incl_space = s
            .lines()
            .filter_map(|line| {
                let mut val_iter = line.chars().array_chunks::<4>();
                let mut vals: Vec<_> = val_iter.by_ref().map(|chunk| chunk[1]).collect();

                if let Some(mut other) = val_iter.into_remainder() {
                    if let Some(val) = other.nth(1) {
                        vals.push(val);
                    }
                }

                vals.get(0)
                    .copied()
                    .and_then(|first_val| if first_val == '1' { None } else { Some(vals) })
            })
            .collect::<Vec<_>>();

        if let Some(last) = vals_incl_space.last() {
            let mut stacks = vec![VecDeque::new(); last.len()];

            for stack_row in vals_incl_space.into_iter() {
                for (i, val) in stack_row.into_iter().enumerate() {
                    if val != ' ' {
                        stacks[i].push_back(val);
                    }
                }
            }
            Ok(Self { stacks: stacks })
        } else {
            Err(StacksParseError::MissingStacks)
        }
    }
}

pub fn get_stacks_and_moves(
    mut input: impl Iterator<Item = impl Into<String>>,
) -> Result<(Stacks, Vec<Move>)> {
    let stacks: Stacks = input
        .by_ref()
        .map(|line| {
            let line: String = line.into();
            line
        })
        .take_while(|line| !line.is_empty())
        .fold(String::new(), |s, l| s + l.as_str() + "\n")
        .parse()
        .map_err(anyhow::Error::from)?;

    let moves = input
        .map(|line| try {
            let line: String = line.into();
            line.parse()?
        })
        .collect::<Result<Vec<Move>>>()?;

    Ok((stacks, moves))
}

pub fn move_and_get_top_9000(input: impl Iterator<Item = impl Into<String>>) -> Result<String> {
    let (mut stacks, moves) = get_stacks_and_moves(input)?;

    for mv in moves.into_iter() {
        stacks.apply_move(mv)?;
    }

    Ok(stacks.top_crates())
}

pub fn move_and_get_top_9001(input: impl Iterator<Item = impl Into<String>>) -> Result<String> {
    let (mut stacks, moves) = get_stacks_and_moves(input)?;

    for mv in moves.into_iter() {
        stacks.apply_move_crate_mover9001(mv)?;
    }

    Ok(stacks.top_crates())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> String {
        r"    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2"
            .to_string()
    }

    #[test]
    fn move_and_get_top_9000_ok() {
        let test_input = test_input();

        let top = move_and_get_top_9000(test_input.lines());

        assert!(top.is_ok());

        assert_eq!(top.unwrap(), "CMZ".to_string());
    }

    #[test]
    fn move_and_get_top_9001_ok() {
        let test_input = test_input();

        let top = move_and_get_top_9001(test_input.lines());

        assert!(top.is_ok());

        assert_eq!(top.unwrap(), "MCD".to_string());
    }
}
