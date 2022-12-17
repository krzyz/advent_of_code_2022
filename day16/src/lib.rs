#![feature(try_blocks)]
#![feature(iter_intersperse)]

use std::collections::{BTreeSet, HashMap};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::alpha1;
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    error::ParseError,
    sequence::tuple,
    IResult,
};
use petgraph::algo::k_shortest_path;
use petgraph::{Graph, Undirected};
use util::{parse_nice, Span};

#[derive(Debug, Clone)]
struct Valve {
    name: String,
    rate: u32,
    leads_to: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    time_left: i32,
    current: usize,
    valves_left: Vec<usize>,
    pressure_built: i32,
    min: i32,
    max: i32,
}
impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.min.partial_cmp(&other.min)
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.min.cmp(&other.min)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Finished(Path),
    Progressing(Path),
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path().partial_cmp(&other.path())
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path().cmp(&other.path())
    }
}

impl State {
    fn path(&self) -> &Path {
        match self {
            Self::Finished(path) => path,
            Self::Progressing(path) => path,
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::Finished(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct ValveGraph {
    rates: Vec<u32>,
    shortest_paths: Vec<HashMap<usize, u32>>,
    starting: usize,
}

impl ValveGraph {
    fn new(valves: Vec<Valve>) -> Result<Self> {
        let rates = valves.iter().map(|valve| valve.rate).collect::<Vec<_>>();

        let inds_map = valves
            .iter()
            .enumerate()
            .map(|(i, valve)| (valve.name.clone(), i as u32))
            .collect::<HashMap<_, _>>();

        let edges = valves
            .into_iter()
            .flat_map(|valve| {
                valve
                    .leads_to
                    .iter()
                    .map(|valve_to| -> Result<(u32, u32)> {
                        let from = inds_map.get(&valve.name).unwrap();
                        inds_map
                            .get(valve_to)
                            .ok_or(anyhow!(format!("Unrecognized valve index: {valve_to}")))
                            .map(|to| (*from, *to))
                    })
                    .filter(|r| r.as_ref().map(|(from, to)| from < to).unwrap_or(true))
                    .collect::<Vec<_>>()
            })
            .collect::<Result<Vec<_>>>()?;
        let graph: Graph<(), (), Undirected> = Graph::from_edges(edges);

        let shortest_paths = (0u32..rates.len() as u32)
            .map(|i| {
                k_shortest_path(&graph, i.into(), None, 1, |_| 1)
                    .iter()
                    .map(|(k, v)| (k.index(), *v as u32))
                    .collect::<HashMap<_, _>>()
            })
            .collect::<Vec<_>>();

        Ok(ValveGraph {
            rates,
            shortest_paths,
            starting: *inds_map.get(&"AA".to_string()).unwrap() as usize,
        })
    }

    fn get_interesting_valves(&self) -> Vec<usize> {
        self.rates
            .iter()
            .enumerate()
            .filter_map(|(i, r)| (*r > 0).then(|| i))
            .collect::<Vec<_>>()
    }

    fn max1(&self, time_left: i32, valves_left: &[usize]) -> i32 {
        let mut rates_left = valves_left
            .iter()
            .map(|i| self.rates[*i])
            .collect::<Vec<_>>();

        rates_left.sort();
        rates_left
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, rate)| (time_left - 2 * i as i32, rate))
            //.inspect(|(i, rate)| println!("in max1: {:#?}", (i, rate)))
            .filter_map(|(t_l, rate)| (t_l > 0).then(|| t_l * rate as i32))
            .sum()
    }

    fn max2(&self, time_left: i32, current: usize, valves_left: &Vec<usize>) -> i32 {
        valves_left
            .iter()
            .map(|&other| {
                let (path_length, next_valves_left, rate) =
                    self.get_next(current, other, valves_left);
                (time_left - path_length - 1).max(0) * rate
                    + self.max1(
                        (time_left - path_length - 1).max(0),
                        next_valves_left.as_slice(),
                    )
            })
            //.inspect(|max| println!("in max2: {max}"))
            .max()
            .unwrap_or(0)
    }

    fn min(&self, time_left: i32, current: usize, valves_left: &Vec<usize>) -> i32 {
        if time_left == 0 || valves_left.is_empty() {
            return 0;
        }

        let (pressure_built, time_left, next, next_valves_left) = valves_left
            .iter()
            .map(|&other| {
                let (path_length, next_valves_left, rate) =
                    self.get_next(current, other, valves_left);
                let next_time_left = (time_left - path_length - 1).max(0);
                (
                    rate * next_time_left,
                    next_time_left,
                    other,
                    next_valves_left,
                )
            })
            .sorted_by(|l, r| (r.0).cmp(&l.0))
            .next()
            .unwrap();

        pressure_built + self.min(time_left, next, &next_valves_left)
    }

    fn get_next(
        &self,
        current: usize,
        other: usize,
        valves_left: &Vec<usize>,
    ) -> (i32, Vec<usize>, i32) {
        let path_length = *self
            .shortest_paths
            .get(current)
            .and_then(|paths| paths.get(&other))
            .unwrap() as i32;
        let mut next_valves_left = valves_left.clone();
        next_valves_left.retain(|&x| x != other);

        (path_length, next_valves_left, self.rates[other] as i32)
    }

    fn make_path(
        &self,
        time_left: i32,
        current: usize,
        valves_left: &Vec<usize>,
        pressure_built: i32,
    ) -> Path {
        Path {
            time_left,
            current,
            valves_left: valves_left.clone(),
            pressure_built,
            min: pressure_built + self.min(time_left, current, valves_left),
            max: pressure_built + self.max2(time_left, current, valves_left),
        }
    }

    fn get_max_pressure(&self) -> i32 {
        let mut paths = std::iter::once(State::Progressing(self.make_path(
            30,
            self.starting,
            &self.get_interesting_valves(),
            0,
        )))
        .collect::<BTreeSet<_>>();

        while paths
            .iter()
            .filter(|path| !path.is_finished())
            .next()
            .is_some()
        {
            //println!("Paths: {paths:#?}");
            let min = paths.iter().rev().next().unwrap().path().min;
            //println!("Best min : {min}");

            paths.retain(|state| state.path().max >= min);

            if let Some(next_considered) = paths
                .iter()
                .rev()
                .filter(|path| !path.is_finished())
                .next()
                .cloned()
            {
                paths.retain(|path| path != &next_considered);

                //println!("Next considered is: {next_considered:#?}");

                let Path {
                    time_left,
                    current,
                    valves_left,
                    pressure_built,
                    ..
                } = next_considered.path();
                for &other in valves_left.iter() {
                    //println!("Now other is {other}");
                    let (path_length, next_valves_left, rate) =
                        self.get_next(*current, other, valves_left);
                    let next_time_left = (time_left - path_length - 1).max(0);
                    let path = self.make_path(
                        next_time_left,
                        other,
                        &next_valves_left,
                        pressure_built + next_time_left * rate,
                    );
                    if next_valves_left.len() == 0 {
                        //println!("Inserting finished: {path:#?}");
                        paths.insert(State::Finished(path))
                    } else if next_time_left > 0 {
                        //println!("Inserting progressing: {path:#?}");
                        paths.insert(State::Progressing(path))
                    } else {
                        //println!("Inserting finished: {path:#?}");
                        paths.insert(State::Finished(path))
                    };
                }
            }
        }

        paths.iter().rev().next().unwrap().path().pressure_built
    }

    fn traverse(
        &self,
        time_left: i32,
        current_pressure: i32,
        pressure_built: i32,
        current: usize,
        valves_left: Vec<usize>,
    ) -> i32 {
        if time_left == 0 {
            return pressure_built;
        }
        if valves_left.is_empty() {
            return pressure_built + time_left * current_pressure;
        }

        valves_left
            .iter()
            .map(|&other| {
                let path_length = *self
                    .shortest_paths
                    .get(current)
                    .and_then(|paths| paths.get(&other))
                    .unwrap() as i32;
                if path_length >= time_left {
                    self.traverse(time_left, current_pressure, pressure_built, current, vec![])
                } else {
                    let mut next_valves_left = valves_left.clone();
                    next_valves_left.retain(|&x| x != other);
                    let new_pressure = current_pressure + self.rates[other] as i32;
                    self.traverse(
                        time_left - (path_length + 1),
                        new_pressure,
                        pressure_built + current_pressure * (path_length + 1),
                        other,
                        next_valves_left,
                    )
                }
            })
            .max()
            .unwrap()
    }
}

fn parse_usize<'a, E>(i: Span<'a>) -> IResult<Span<'a>, u32, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map_res(digit1, |i: Span<'a>| FromStr::from_str(i.fragment()))(i)
}

fn parse_valves_names<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Vec<&str>, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    separated_list0(tag(", "), map(alpha1, |s: Span<'a>| *s.fragment()))(i)
}

fn parse_valve<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Valve, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, std::num::ParseIntError>,
{
    map(
        tuple((
            preceded(tag("Valve "), alpha1),
            preceded(tag(" has flow rate="), parse_usize),
            preceded(
                alt((
                    tag("; tunnels lead to valves "),
                    tag("; tunnel leads to valve "),
                )),
                parse_valves_names,
            ),
        )),
        |(name, rate, valves)| Valve {
            name: name.to_string(),
            rate,
            leads_to: valves
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        },
    )(i)
}

fn parse_valves(input: impl Iterator<Item = String>) -> Result<Vec<Valve>> {
    input
        .map(|l| -> Result<Valve> {
            parse_nice(l.as_str(), parse_valve).ok_or(anyhow!("Couldn't parse line!"))
        })
        .collect::<Result<Vec<_>>>()
}

pub fn get_max_pressure(input: impl Iterator<Item = String>) -> Result<i32> {
    let valves = parse_valves(input)?;
    let valve_graph = ValveGraph::new(valves)?;
    Ok(valve_graph.traverse(
        30,
        0,
        0,
        valve_graph.starting,
        valve_graph.get_interesting_valves(),
    ))
}

pub fn get_max_pressure_2(input: impl Iterator<Item = String>) -> Result<i32> {
    let valves = parse_valves(input)?;
    let valve_graph = ValveGraph::new(valves)?;
    Ok(valve_graph.get_max_pressure())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_max_pressure(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1651);
    }

    #[test]
    fn part2() {
        let res = get_max_pressure_2(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1651);
    }
}
