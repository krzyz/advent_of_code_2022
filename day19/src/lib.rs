#![feature(try_blocks)]
#![feature(iter_intersperse)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use nom::character::complete::{alpha1, multispace1};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::{
    bytes::complete::tag,
    combinator::{map, map_res},
    error::ParseError,
    sequence::tuple,
    IResult,
};
use num_derive::FromPrimitive;
use strum::EnumCount;
use strum_macros::{Display, EnumCount, EnumString};
use util::{parse_nice, parse_number, Span};

#[derive(
    Debug,
    Copy,
    Clone,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    EnumCount,
    FromPrimitive,
)]
enum Material {
    #[strum(ascii_case_insensitive)]
    Ore = 0,
    #[strum(ascii_case_insensitive)]
    Clay,
    #[strum(ascii_case_insensitive)]
    Obsidian,
    #[strum(ascii_case_insensitive)]
    Geode,
}

impl Material {
    fn next(&self) -> Self {
        match *self {
            Material::Ore => Material::Clay,
            Material::Clay => Material::Obsidian,
            Material::Obsidian => Material::Geode,
            Material::Geode => Material::Geode,
        }
    }
}

static MATERIALS: [Material; Material::COUNT] = [
    Material::Ore,
    Material::Clay,
    Material::Obsidian,
    Material::Geode,
];

#[derive(Debug)]
struct Blueprint {
    costs: BTreeMap<Material, BTreeMap<Material, i32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
struct State {
    minute: i32,
    materials: [i32; Material::COUNT],
    incomes: [i32; Material::COUNT],
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let calc_score = |s: &State| {
            (0..4)
                .map(|i| {
                    10f32.powf(i as f32 * 3.0)
                        * (s.materials[i] as f32 + s.incomes[i] as f32 * (24 - s.minute) as f32)
                })
                .sum::<f32>()
        };

        let self_score = calc_score(self);
        let other_score = calc_score(other);
        let cmp = self_score.partial_cmp(&other_score);
        if let Some(cmp) = cmp {
            if !cmp.is_eq() {
                return cmp;
            }
        }

        (self.minute, self.materials, self.incomes).cmp(&(
            other.minute,
            other.materials,
            other.incomes,
        ))
    }
}

impl State {
    fn new() -> Self {
        let mut incomes = [0; Material::COUNT];
        incomes[Material::Ore as usize] = 1;

        State {
            minute: 0,
            materials: [0; Material::COUNT],
            incomes,
        }
    }

    fn progress(mut self) -> Self {
        for (inc_m_i, inc) in self.incomes.iter().enumerate() {
            self.materials[inc_m_i] += inc;
        }

        State {
            minute: self.minute + 1,
            materials: self.materials,
            incomes: self.incomes,
        }
    }

    fn build_robot(mut self, blueprint: &Blueprint, material: Material) -> Self {
        for (cost_m, c) in blueprint.costs[&material].iter() {
            self.materials[*cost_m as usize] -= c;
        }

        self.incomes[material as usize] += 1;

        self
    }

    fn turns_to(&self, blueprint: &Blueprint, material: Material) -> Option<f32> {
        blueprint.costs[&material]
            .iter()
            .map(|(cost_m, cost)| -> Option<f32> {
                let income = self.incomes[*cost_m as usize];
                (income > 0).then(|| {
                    -(*cost as f32 - self.materials[*cost_m as usize] as f32).max(0.0)
                        / self.incomes[*cost_m as usize] as f32
                })
            })
            .collect::<Option<Vec<_>>>()?
            .iter()
            .copied()
            .sorted_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .next()
    }

    fn is_build_desirable(&self, blueprint: &Blueprint, m: &Material) -> bool {
        match m {
            Material::Geode => true,
            Material::Obsidian | Material::Clay => {
                (self.incomes[*m as usize] as f32 / self.incomes[Material::Ore as usize] as f32)
                    < (blueprint.costs[&m.next()][m] as f32
                        / blueprint.costs[&m.next()][&Material::Ore] as f32)
            }
            Material::Ore => true,
        }
    }

    fn get_best_next(states: Vec<State>, blueprint: &Blueprint) -> State {
        //println!("\n\n\nstart inspect\n\n\n");
        let mut sorted_by_turns = states
            .into_iter()
            .flat_map(|state| {
                let available = MATERIALS
                    .iter()
                    .filter(|m| {
                        blueprint.costs[m]
                            .iter()
                            .all(|(cost_m, cost)| cost <= &state.materials[*cost_m as usize])
                    })
                    .map(|m| Some(m))
                    .chain(std::iter::once(None))
                    .collect::<BTreeSet<_>>();

                let new_state = state.progress();

                available
                    .iter()
                    .map(move |&m_to_build| {
                        let new_state_considered = if let Some(m_to_build) = m_to_build {
                            new_state.clone().build_robot(blueprint, *m_to_build)
                        } else {
                            new_state.clone()
                        };

                        (
                            new_state_considered,
                            m_to_build,
                            MATERIALS
                                .iter()
                                .rev()
                                .map(|m| new_state.turns_to(blueprint, *m))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .sorted_by(|(s1, ma, a), (_, mb, b)| {
                //println!("For minute {} Comparing {a:#?} with {b:#?}", s1.minute);
                //println!("Result: {:#?}", a.partial_cmp(b));
                if ma == &Some(&Material::Geode) {
                    Ordering::Greater
                } else if mb == &Some(&Material::Geode) {
                    Ordering::Less
                } else {
                    a.partial_cmp(b).unwrap_or(Ordering::Equal)
                }
            })
            .inspect(|(s, m, t)| println!("For state: {s:#?}, built: {m:#?}, turns: {t:#?}\n\n"))
            .collect::<Vec<_>>();

        //println!("\n\n\nEnd inspect\n\n\n\n\n\n");

        let mut best_states_with_turns: Vec<(State, Option<&Material>, Vec<Option<f32>>)> = vec![];
        while let Some((state, built_m, turns)) = sorted_by_turns.pop() {
            if let Some((_, _, best_turns)) = best_states_with_turns.first() {
                if best_turns == &turns && state.minute < 24 {
                    best_states_with_turns.push((state, built_m, turns))
                }
            } else {
                best_states_with_turns.push((state, built_m, turns))
            }
        }

        let solved = if best_states_with_turns.len() == 1 {
            true
        } else {
            if let Some(build_material) = best_states_with_turns.first().unwrap().1 {
                if build_material == &Material::Geode {
                    true
                } else {
                    false
                }
            } else if let Some(turns_to_build_geode) = best_states_with_turns.first().unwrap().2[0]
            {
                if turns_to_build_geode > -1.0 {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if solved {
            best_states_with_turns.pop().unwrap().0
        } else {
            State::get_best_next(
                best_states_with_turns
                    .into_iter()
                    .map(|(state, _, _)| state)
                    .collect::<Vec<_>>(),
                blueprint,
            )
        }
    }

    fn get_next(&self, blueprint: &Blueprint) -> Vec<State> {
        let available = MATERIALS.iter().filter(|m| {
            blueprint.costs[m]
                .iter()
                .all(|(cost_m, cost)| cost <= &self.materials[*cost_m as usize])
        });

        available
            .map(|m| (Some(m), self.clone()))
            //            .inspect(|(m, _)| println!("  Available m: {}", m.unwrap()))
            .chain(std::iter::once((None, self.clone())))
            .map(|(m, state)| (m, state.progress()))
            .map(|(m, mut state)| {
                if let Some(m) = m {
                    state = state.build_robot(blueprint, *m)
                }

                state
            })
            //            .inspect(|s| println!("{s:#?}"))
            .collect()
    }
}

fn parse_material<'a, E>(i: Span<'a>) -> IResult<Span<'a>, Material, E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, anyhow::Error>,
{
    map_res(alpha1, |m: Span<'a>| {
        FromStr::from_str(m.fragment()).map_err(anyhow::Error::msg)
    })(i)
}

fn parse_robot_cost<'a, E>(i: Span<'a>) -> IResult<Span<'a>, (Material, BTreeMap<Material, i32>), E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, anyhow::Error>,
{
    map(
        tuple((
            preceded(tag("Each "), parse_material),
            delimited(
                tag(" robot costs "),
                separated_list0(
                    tag(" and "),
                    separated_pair(parse_number, tag(" "), parse_material),
                ),
                tag("."),
            ),
        )),
        |(m, costs_vec)| {
            (
                m,
                costs_vec.into_iter().map(|(i, m)| (m, i as i32)).collect(),
            )
        },
    )(i)
}

fn parse_blueprint<'a, E>(i: Span<'a>) -> IResult<Span<'a>, (i32, Blueprint), E>
where
    E: ParseError<Span<'a>> + nom::error::FromExternalError<Span<'a>, anyhow::Error>,
{
    map(
        pair(
            delimited(tag("Blueprint "), parse_number, pair(tag(":"), multispace1)),
            separated_list0(multispace1, parse_robot_cost),
        ),
        |(i, robot_costs_vec)| {
            (
                i as i32,
                Blueprint {
                    costs: robot_costs_vec.into_iter().collect(),
                },
            )
        },
    )(i)
}

fn parse_blueprints(input: &str) -> Result<BTreeMap<i32, Blueprint>> {
    Ok(
        parse_nice(input, separated_list0(multispace1, parse_blueprint))
            .ok_or(anyhow!("Error parsing input"))?
            .into_iter()
            .collect(),
    )
}

fn get_blueprint_quality(blueprint: &Blueprint) -> u32 {
    let mut states = BTreeSet::from_iter([State::new()]);
    let mut max_geodes = 0;

    while !states.is_empty() {
        //println!("Num states: {}, max_geodes: {max_geodes}", states.len());
        let next_state = states.pop_last().unwrap();

        let geode_count = next_state.materials[Material::Geode as usize];
        //println!("count: {geode_count}");
        println!("Next state: {next_state:#?}, count: {geode_count}");
        if next_state.minute == 24 {
            max_geodes = max_geodes.max(geode_count);
        } else {
            //states.extend(next_state.get_next(blueprint))
            states.insert(State::get_best_next(vec![next_state], blueprint));
        }
    }

    max_geodes as u32
}

pub fn get_sum_quality_levels(input: &str) -> Result<u32> {
    let blueprints = parse_blueprints(input)?;

    for (_, blueprint) in blueprints.iter() {
        println!("{}", get_blueprint_quality(blueprint))
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_sum_quality_levels(TEST_INPUT);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 33);
    }
}
