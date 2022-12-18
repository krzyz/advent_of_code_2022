#![feature(int_roundings)]
#![feature(try_blocks)]

use std::collections::{BTreeSet, HashMap, HashSet};

use anyhow::{anyhow, Result};
use itertools::iproduct;

static DIRECTIONS: [(i32, i32, i32); 6] = [
    (-1, 0, 0),
    (1, 0, 0),
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
];

pub fn get_num_exposed_sides(input: impl Iterator<Item = String>) -> Result<u64> {
    let mut exposed_sides = 0;
    let mut cubes: HashSet<(i32, i32, i32)> = HashSet::new();

    for loc in input.map(parse_location) {
        let loc = loc?;

        let neighbours = DIRECTIONS
            .iter()
            .filter_map(|d| {
                let neighbor_loc = (loc.0 + d.0, loc.1 + d.1, loc.2 + d.2);
                cubes.contains(&neighbor_loc).then_some(())
            })
            .count() as u64;

        exposed_sides += 6 - 2 * neighbours;

        cubes.insert(loc);
    }

    Ok(exposed_sides)
}

pub fn get_num_exposed_sides_2(input: impl Iterator<Item = String>, only_out: bool) -> Result<i64> {
    let cubes = input.map(parse_location).collect::<Result<HashSet<_>>>()?;

    let ranges = cubes
        .iter()
        .fold(None, |acc, loc| match acc {
            None => Some(((loc.0, loc.0), (loc.1, loc.1), (loc.2, loc.2))),
            Some(acc) => Some((
                (acc.0 .0.min(loc.0 - 1), acc.0 .1.max(loc.0 + 1)),
                (acc.1 .0.min(loc.1 - 1), acc.1 .1.max(loc.1 + 1)),
                (acc.2 .0.min(loc.2 - 1), acc.2 .1.max(loc.2 + 1)),
            )),
        })
        .ok_or(anyhow!("No cubes!"))?;

    let mut empty_space = iproduct!(
        ranges.0 .0..=ranges.0 .1,
        ranges.1 .0..=ranges.1 .1,
        ranges.2 .0..=ranges.2 .1
    )
    .filter(|loc| !cubes.contains(loc))
    .collect::<BTreeSet<_>>();

    let mut exposed_sides = 0;

    while !empty_space.is_empty() {
        let mut out = false;
        let next = *empty_space.iter().next().unwrap();
        empty_space.remove(&next);

        let mut exposed_sides_pocket = 0;
        let mut connections = HashMap::from([(next, vec![])]);

        while !connections.is_empty() {
            let (to, from) = connections.iter().next().unwrap();
            let (to, from) = (*to, from.clone());

            empty_space.remove(&to);
            connections.remove(&to);

            if !(ranges.0 .0..=ranges.0 .1).contains(&to.0)
                || !(ranges.1 .0..=ranges.1 .1).contains(&to.1)
                || !(ranges.2 .0..=ranges.2 .1).contains(&to.2)
            {
                out = true;
            } else if cubes.contains(&to) {
                exposed_sides_pocket += from.len() as i64;
            } else {
                for next in DIRECTIONS
                    .iter()
                    .map(|d| (to.0 + d.0, to.1 + d.1, to.2 + d.2))
                    .filter_map(|next| (!from.contains(&next)).then_some(next))
                {
                    connections
                        .entry(next)
                        .and_modify(|from| from.push(to))
                        .or_insert(vec![to]);
                }
            }
        }

        if out || !only_out {
            exposed_sides += exposed_sides_pocket;
        }
    }

    Ok(exposed_sides)
}

fn parse_location(l: String) -> Result<(i32, i32, i32)> {
    let v: [i32; 3] = l
        .split(',')
        .map(|n| n.parse::<i32>().map_err(anyhow::Error::msg))
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| anyhow!("Too many elements"))?;
    Ok((v[0], v[1], v[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_num_exposed_sides(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 64);
    }

    #[test]
    fn part1b() {
        let res = get_num_exposed_sides_2(TEST_INPUT.lines().map(|l| l.to_string()), false);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 64);
    }

    #[test]
    fn part2() {
        let res = get_num_exposed_sides_2(TEST_INPUT.lines().map(|l| l.to_string()), true);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 58);
    }
}
