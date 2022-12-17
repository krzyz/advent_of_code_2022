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
    times_left: Vec<i32>,
    currents: Vec<usize>,
    valves_left: Vec<usize>,
    histories: Vec<Vec<(i32, usize)>>,
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

    fn max1(&self, times_left: &[i32], valves_left: &[usize]) -> i32 {
        let mut rates_left = valves_left
            .iter()
            .map(|i| self.rates[*i])
            .collect::<Vec<_>>();

        let times_left_iter = times_left
            .iter()
            .flat_map(|&t| (0..).map(move |i| t - 2 * i).take_while(|&t| t > 0))
            .sorted()
            .rev();

        rates_left.sort();
        rates_left
            .into_iter()
            .rev()
            .zip(times_left_iter)
            .filter_map(|(rate, t_l)| (t_l > 0).then(|| t_l * rate as i32))
            .sum()
    }

    fn max2(
        &self,
        times_left: &[i32],
        currents: &[usize],
        valves_left: &[usize],
        level: i32,
    ) -> i32 {
        valves_left
            .iter()
            .flat_map(|&other| {
                self.get_many_next(times_left, currents, other, valves_left, None)
                    .map(
                        |(pressure, new_times_left, new_currents, next_valves_left, _)| {
                            if level > 0 {
                                pressure
                                    + self.max2(
                                        new_times_left.as_slice(),
                                        new_currents.as_slice(),
                                        next_valves_left.as_slice(),
                                        level - 1,
                                    )
                            } else {
                                pressure
                                    + self.max1(
                                        new_times_left.as_slice(),
                                        next_valves_left.as_slice(),
                                    )
                            }
                        },
                    )
            })
            .max()
            .unwrap_or(0)
    }

    fn min(&self, times_left: &[i32], currents: &[usize], valves_left: &[usize]) -> i32 {
        if times_left.iter().all(|&t| t == 0) || valves_left.is_empty() {
            return 0;
        }

        let (pressure_built, times_left, next, next_valves_left, _) = valves_left
            .iter()
            .flat_map(|&other| self.get_many_next(times_left, currents, other, valves_left, None))
            .sorted_by(|l, r| (r.0).cmp(&l.0))
            .next()
            .unwrap();

        pressure_built + self.min(times_left.as_slice(), next.as_slice(), &next_valves_left)
    }

    fn get_many_next<'a>(
        &'a self,
        times_left: &'a [i32],
        currents: &'a [usize],
        other: usize,
        valves_left: &'a [usize],
        histories: Option<&'a [Vec<(i32, usize)>]>,
    ) -> impl Iterator<
        Item = (
            i32,
            Vec<i32>,
            Vec<usize>,
            Vec<usize>,
            Option<Vec<Vec<(i32, usize)>>>,
        ),
    > + 'a {
        currents
            .iter()
            .zip(times_left.iter())
            .enumerate()
            .filter(|(_, (_, time_left))| **time_left != 0)
            .unique()
            .map(move |(i, (current, time_left))| {
                let (path_length, next_valves_left, rate) =
                    self.get_next(*current, other, valves_left);
                let next_time_left = (time_left - path_length - 1).max(0);
                let mut new_currents = currents.to_vec();
                *new_currents.get_mut(i).unwrap() = other;
                let mut new_times_left = times_left.to_vec();
                *new_times_left.get_mut(i).unwrap() = next_time_left;
                let new_histories = histories.map(|histories| {
                    let mut new_histories = histories.to_vec();
                    new_histories
                        .get_mut(i)
                        .unwrap()
                        .push((next_time_left, other));
                    new_histories
                });
                (
                    rate * next_time_left,
                    new_times_left,
                    new_currents,
                    next_valves_left,
                    new_histories,
                )
            })
    }

    fn get_next(
        &self,
        current: usize,
        other: usize,
        valves_left: &[usize],
    ) -> (i32, Vec<usize>, i32) {
        let path_length = *self
            .shortest_paths
            .get(current)
            .and_then(|paths| paths.get(&other))
            .unwrap() as i32;
        let mut next_valves_left = valves_left.to_vec();
        next_valves_left.retain(|&x| x != other);

        (path_length, next_valves_left, self.rates[other] as i32)
    }

    fn make_path(
        &self,
        times_left: Vec<i32>,
        currents: Vec<usize>,
        mut histories: Vec<Vec<(i32, usize)>>,
        valves_left: Vec<usize>,
        pressure_built: i32,
    ) -> Path {
        if histories.len() == 0 {
            histories = times_left
                .iter()
                .zip(currents.iter())
                .map(|(t, c)| vec![(*t, *c)])
                .collect::<Vec<_>>();
        }

        let min = pressure_built
            + self.min(
                times_left.as_slice(),
                currents.as_slice(),
                valves_left.as_slice(),
            );
        let max = pressure_built
            + self.max2(
                times_left.as_slice(),
                currents.as_slice(),
                valves_left.as_slice(),
                0,
            );
        Path {
            times_left,
            currents,
            valves_left,
            histories,
            pressure_built,
            min,
            max,
        }
    }

    fn get_max_pressure(&self, num_agents: usize, time: i32) -> i32 {
        let mut paths = std::iter::once(State::Progressing(self.make_path(
            vec![time; num_agents],
            vec![self.starting, self.starting],
            vec![],
            self.get_interesting_valves(),
            0,
        )))
        .collect::<BTreeSet<_>>();

        let mut i = 0;

        while paths
            .iter()
            .filter(|path| !path.is_finished())
            .next()
            .is_some()
        {
            let min = paths.iter().rev().next().unwrap().path().min;
            paths.retain(|state| state.path().max >= min);

            if let Some(next_considered) = paths
                .iter()
                .rev()
                .filter(|path| !path.is_finished())
                .next()
                .cloned()
            {
                paths.retain(|path| path != &next_considered);

                let Path {
                    times_left,
                    currents,
                    valves_left,
                    pressure_built,
                    histories,
                    ..
                } = next_considered.path();
                for &other in valves_left.iter() {
                    for (pressure, new_times_left, new_currents, next_valves_left, new_histories) in
                        self.get_many_next(
                            times_left,
                            currents,
                            other,
                            valves_left,
                            Some(histories.as_slice()),
                        )
                    {
                        let next_valves_len = next_valves_left.len();
                        let any_times_left = new_times_left.iter().any(|&t| t > 0);

                        let path = self.make_path(
                            new_times_left,
                            new_currents,
                            new_histories.unwrap(),
                            next_valves_left,
                            pressure_built + pressure,
                        );
                        if next_valves_len == 0 {
                            paths.insert(State::Finished(path))
                        } else if any_times_left {
                            paths.insert(State::Progressing(path))
                        } else {
                            paths.insert(State::Finished(path))
                        };
                    }
                }
            }
            i += 1;
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

pub fn get_max_pressure_2(
    input: impl Iterator<Item = String>,
    num_agents: usize,
    time: i32,
) -> Result<i32> {
    let valves = parse_valves(input)?;
    let valve_graph = ValveGraph::new(valves)?;
    Ok(valve_graph.get_max_pressure(num_agents, time))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1old() {
        let res = get_max_pressure(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1651);
    }

    #[test]
    fn part1() {
        let res = get_max_pressure_2(TEST_INPUT.lines().map(|l| l.to_string()), 1, 30);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1651);
    }

    #[test]
    fn part2() {
        let res = get_max_pressure_2(TEST_INPUT.lines().map(|l| l.to_string()), 2, 26);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1707);
    }
}
