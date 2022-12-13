#![feature(try_blocks)]
#![feature(iter_intersperse)]

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use miette::GraphicalReportHandler;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    error::ParseError,
    multi::separated_list0,
    sequence::delimited,
    IResult,
};
use nom_locate::LocatedSpan;
use nom_supreme::{
    error::{BaseErrorKind, ErrorTree, GenericErrorTree},
    final_parser::final_parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum List {
    Integer(i32),
    List(Vec<List>),
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            List::Integer(i) => write!(f, "{i}"),
            List::List(v) => write!(
                f,
                "[{}]",
                itertools::Itertools::intersperse(
                    v.iter().map(|item| format!("{item}")),
                    ",".to_string()
                )
                .collect::<String>()
            ),
        }
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (&List::Integer(l), &List::Integer(r)) => l.cmp(&r),
            (List::List(l), List::List(r)) => l
                .iter()
                .zip(r.iter())
                .inspect(
                    #[allow(unused)]
                    |(l, r)| {
                        #[cfg(debug_assertions)]
                        println!("Comparing {l} with {r}")
                    },
                )
                .filter_map(|(l, r)| match l.cmp(r) {
                    Ordering::Equal => None,
                    ord => Some(ord),
                })
                .inspect(
                    #[allow(unused)]
                    |l| {
                        #[cfg(debug_assertions)]
                        println!("got {l:#?}")
                    },
                )
                .next()
                .unwrap_or_else(|| l.len().cmp(&r.len())),
            (&List::Integer(l), &List::List(_)) => List::List(vec![List::Integer(l)]).cmp(other),
            (&List::List(_), &List::Integer(r)) => self.cmp(&List::List(vec![List::Integer(r)])),
        }
    }
}

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("bad input")]
struct BadInput<'a> {
    #[source_code]
    src: &'a str,

    #[label("{kind}")]
    bad_bit: miette::SourceSpan,

    kind: BaseErrorKind<&'a str, Box<dyn std::error::Error + Send + Sync>>,
}

fn parse_list<'a, E>(i: Span<'a>) -> IResult<Span<'a>, List, E>
where
    E: ParseError<Span<'a>>
        + nom::error::FromExternalError<nom_locate::LocatedSpan<&'a str>, std::num::ParseIntError>,
{
    alt((
        map_res(digit1, |s: Span<'a>| {
            FromStr::from_str(s.fragment()).map(|n| List::Integer(n))
        }),
        map(
            delimited(tag("["), separated_list0(tag(","), parse_list), tag("]")),
            |l| List::List(l),
        ),
    ))(i)
}

fn parse_line(l: String) -> Option<List> {
    let line_span = Span::new(l.as_str());
    let line: Result<_, ErrorTree<Span>> = final_parser(parse_list::<ErrorTree<Span>>)(line_span);
    match line {
        Ok(line) => Some(line),
        Err(e) => {
            match e {
                GenericErrorTree::Base { location, kind } => {
                    let offset = location.location_offset().into();
                    let err = BadInput {
                        src: l.as_str(),
                        bad_bit: miette::SourceSpan::new(offset, 0.into()),
                        kind,
                    };
                    let mut s = String::new();
                    GraphicalReportHandler::new()
                        .render_report(&mut s, &err)
                        .unwrap();
                    println!("{s}");
                }
                GenericErrorTree::Stack { .. } => todo!("stack"),
                GenericErrorTree::Alt(_) => todo!("alt"),
            }
            None
        }
    }
}

pub fn get_pair_list_iter(
    input: impl Iterator<Item = String>,
) -> Result<impl Iterator<Item = (List, List)>> {
    Ok(input
        .chunks(3)
        .into_iter()
        .map(|chunk| {
            chunk
                .filter_map(|l| if !l.is_empty() { parse_line(l) } else { None })
                .collect::<Vec<_>>()
        })
        .map(|list_vec| -> Result<(List, List)> {
            let mut list_vec = list_vec.clone();
            let r = list_vec.pop().ok_or(anyhow!("Missing entry in input"))?;
            let l = list_vec
                .pop()
                .ok_or(anyhow!("Less than two entires in input!"))?;
            Ok((l, r))
        })
        .collect::<Result<Vec<(List, List)>>>()?
        .into_iter())
}

pub fn get_sum_right_order(input: impl Iterator<Item = String>) -> Result<usize> {
    Ok(get_pair_list_iter(input)?
        .enumerate()
        .inspect(
            #[allow(unused)]
            |(i, (l, r))| {
                #[cfg(debug_assertions)]
                println!("== Pair {} == l.cmp(r) is  {:#?}", i + 1, l.cmp(r));
            },
        )
        .filter_map(|(i, (l, r))| (l <= r).then(|| i + 1))
        .sum())
}

pub fn get_decoder_key(input: impl Iterator<Item = String>) -> Result<usize> {
    let divider_packets = [
        List::List(vec![List::List(vec![List::Integer(2)])]),
        List::List(vec![List::List(vec![List::Integer(6)])]),
    ];

    Ok(get_pair_list_iter(input)?
        .map(|(l, r)| [l, r])
        .flatten()
        .chain(divider_packets.iter().cloned())
        .sorted()
        .inspect(
            #[allow(unused)]
            |l| {
                #[cfg(debug_assertions)]
                println!("{l}");
            },
        )
        .enumerate()
        .filter(|list| divider_packets.iter().contains(&(list.1)))
        .map(|(i, _)| i + 1)
        .product())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_sum_right_order(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 13);
    }

    #[test]
    fn part2() {
        let res = get_decoder_key(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 140);
    }
}
