use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::alpha1,
        complete::{char, multispace0},
        complete::{digit1, line_ending},
    },
    combinator::{all_consuming, map, map_res, verify},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, terminated, tuple},
    Finish, IResult,
};
use num::{integer::div_floor, Integer};

#[derive(Debug, Clone, Copy)]
pub enum OpRhs {
    Int(i32),
    Old,
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Multiply(OpRhs),
    Add(OpRhs),
}

#[derive(Debug, Clone, Copy)]
pub struct Test {
    divisible_by: i32,
    true_pass_to: usize,
    false_pass_to: usize,
}

#[derive(Debug, Clone)]
pub struct Monkey {
    items: Vec<i32>,
    operation: Operation,
    test: Test,
}

fn parse_pass_to(condition_match_value: bool, i: &str) -> IResult<&str, usize> {
    preceded(
        multispace0,
        preceded(
            verify(
                map_res(preceded(tag("If "), alpha1), <bool as FromStr>::from_str),
                |s: &bool| s == &condition_match_value,
            ),
            map_res(
                preceded(tag(": throw to monkey "), digit1),
                FromStr::from_str,
            ),
        ),
    )(i)
}

fn parse_test(i: &str) -> IResult<&str, Test> {
    preceded(
        multispace0,
        map(
            tuple((
                map_res(
                    preceded(tag("Test: divisible by "), digit1),
                    FromStr::from_str,
                ),
                |j| parse_pass_to(true, j),
                |j| parse_pass_to(false, j),
            )),
            |(divisible_by, true_pass_to, false_pass_to)| Test {
                divisible_by,
                true_pass_to,
                false_pass_to,
            },
        ),
    )(i)
}

fn parse_op_rhs(i: &str) -> IResult<&str, OpRhs> {
    preceded(
        multispace0,
        alt((
            map(map_res(digit1, FromStr::from_str), OpRhs::Int),
            map_res(alpha1, |s: &str| {
                if s == "old" {
                    Ok(OpRhs::Old)
                } else {
                    Err(anyhow!("Unrecognizable operation right hand size: {s}"))
                }
            }),
        )),
    )(i)
}

fn parse_operation(i: &str) -> IResult<&str, Operation> {
    preceded(
        multispace0,
        map_res(
            tuple((
                preceded(tag("Operation: new = old "), alt((char('*'), char('+')))),
                parse_op_rhs,
            )),
            |(op, num)| match op {
                '*' => Ok(Operation::Multiply(num)),
                '+' => Ok(Operation::Add(num)),
                _ => Err(anyhow!("Unrecognized binary operator: {op}")),
            },
        ),
    )(i)
}

fn parse_starting_items(i: &str) -> IResult<&str, Vec<i32>> {
    delimited(
        multispace0,
        preceded(
            tag("Starting items: "),
            separated_list0(tag(", "), map_res(digit1, FromStr::from_str)),
        ),
        line_ending,
    )(i)
}

fn parse_monkey(i: &str) -> IResult<&str, (Monkey, usize)> {
    preceded(
        multispace0,
        map(
            tuple((
                delimited(tag("Monkey "), map_res(digit1, FromStr::from_str), tag(":")),
                parse_starting_items,
                parse_operation,
                parse_test,
            )),
            |(i, items, operation, test)| {
                (
                    Monkey {
                        items,
                        operation,
                        test,
                    },
                    i,
                )
            },
        ),
    )(i)
}

pub fn get_monkey_business(input: &str, rounds: usize) -> Result<i32> {
    let monkeys = all_consuming(terminated(many0(parse_monkey), multispace0))(input)
        .finish()
        .unwrap()
        .1;

    let mut monkeys = monkeys
        .into_iter()
        .enumerate()
        .map(|(i_enumerate, (monkey, i_parse))| {
            if i_enumerate == i_parse {
                Ok(monkey)
            } else {
                Err(anyhow!("Monkeys are not listed in order!"))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let mut times_inspected_per_monkey = vec![0; monkeys.len()];

    for _ in 0..rounds {
        for i in 0..monkeys.len() {
            let mut current_monkey = monkeys.get(i).unwrap().clone();

            let current_inspected = times_inspected_per_monkey.get_mut(i).unwrap();

            while let Some(worry_lvl) = current_monkey.items.pop() {
                let new_worry_lvl = match current_monkey.operation {
                    Operation::Multiply(x) => match x {
                        OpRhs::Int(y) => worry_lvl * y,
                        OpRhs::Old => worry_lvl * worry_lvl,
                    },
                    Operation::Add(x) => match x {
                        OpRhs::Int(y) => worry_lvl + y,
                        OpRhs::Old => 2 * worry_lvl,
                    },
                };
                let new_worry_lvl = div_floor(new_worry_lvl, 3);

                let throw_to = if new_worry_lvl.is_multiple_of(&current_monkey.test.divisible_by) {
                    current_monkey.test.true_pass_to
                } else {
                    current_monkey.test.false_pass_to
                };

                monkeys
                    .get_mut(throw_to)
                    .context(format!("Trying to throw to nonexisting Monkey {i}"))?
                    .items
                    .push(new_worry_lvl);

                *current_inspected += 1;
            }

            *monkeys.get_mut(i).unwrap() = current_monkey;
        }
    }

    times_inspected_per_monkey.sort();

    Ok(times_inspected_per_monkey.iter().rev().take(2).product())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_monkey_business(TEST_INPUT, 20);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 10605);
    }
}
