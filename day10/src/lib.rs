#![feature(try_blocks)]
#![feature(get_many_mut)]

use std::iter;
use std::str::FromStr;

use anyhow::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{complete::char, complete::digit1},
    combinator::{all_consuming, map, map_res, opt, recognize},
    sequence::{preceded, tuple},
    Finish, IResult,
};

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Addx(i32),
    Noop,
}

fn parse_addx(i: &str) -> IResult<&str, Instruction> {
    map(
        preceded(
            tag("addx "),
            map_res(
                recognize(tuple((opt(char('-')), digit1))),
                FromStr::from_str,
            ),
        ),
        |n| Instruction::Addx(n),
    )(i)
}

fn parse_noop(i: &str) -> IResult<&str, Instruction> {
    map(tag("noop"), |_| Instruction::Noop)(i)
}

fn parse_instruction(i: &str) -> IResult<&str, Instruction> {
    alt((parse_addx, parse_noop))(i)
}

pub fn get_instructions(input: impl Iterator<Item = String>) -> impl Iterator<Item = Instruction> {
    input.map(|l| {
        all_consuming(parse_instruction)(l.as_str())
            .finish()
            .unwrap()
            .1
    })
}

pub fn get_sum_signal_strengths(
    input: impl Iterator<Item = String>,
    first_cycle: usize,
    cycle_size: usize,
    num_signals: usize,
) -> Result<i32> {
    let sum: i32 = get_instructions(input)
        .flat_map(|i| match i {
            Instruction::Noop => vec![None],
            Instruction::Addx(n) => vec![None, Some(n)],
        })
        .scan(1, |v, new_v| {
            if let Some(new_v) = new_v {
                *v += new_v;
            }
            Some(*v)
        })
        .enumerate()
        .map(|(i, x)| (i + 2, x))
        .skip(first_cycle - 2)
        .step_by(cycle_size)
        .take(num_signals)
        .map(|(i, x)| i as i32 * x)
        .sum();

    Ok(sum)
}

pub fn get_crt_output(input: impl Iterator<Item = String>, rows: usize, cols: usize) -> String {
    iter::once(None)
        .chain(get_instructions(input).flat_map(|i| match i {
            Instruction::Noop => vec![None],
            Instruction::Addx(n) => vec![None, Some(n)],
        }))
        .scan(1, |v, new_v| {
            if let Some(new_v) = new_v {
                *v += new_v;
            }
            Some(*v)
        })
        .enumerate()
        .map(|(i, x)| {
            let c = if (x - (i % cols) as i32).abs() <= 1 {
                '#'
            } else {
                '.'
            };
            (i, c)
        })
        .take_while(|(i, _)| *i < rows * cols)
        .flat_map(|(i, c)| {
            if (i + 1) % cols == 0 && i != rows * cols - 1 {
                vec![c, '\n']
            } else {
                vec![c]
            }
        })
        .collect()
}

// Added after checking answers on reddit :)
pub fn prettify(output: String) -> String {
    output.replace("#", "██").replace(".", "░░")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");
    const TEST_2_OUTPUT: &str = include_str!("../data/test_2_output");

    #[test]
    fn part1() {
        let res = get_sum_signal_strengths(TEST_INPUT.lines().map(|l| l.to_string()), 20, 40, 6);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 13140);
    }

    #[test]
    fn part2() {
        let output = get_crt_output(TEST_INPUT.lines().map(|l| l.to_string()), 6, 40);
        assert_eq!(output.as_str(), TEST_2_OUTPUT);
    }
}
