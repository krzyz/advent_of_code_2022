#![feature(try_blocks)]

use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::collections::BTreeSet;
use std::io::{self, BufRead};

fn get_value(c: char) -> Option<i32> {
    match c as u32 {
        i if (65..=90).contains(&i) => Some((i - 38) as i32),
        i if (97..=122).contains(&i) => Some((i - 96) as i32),
        _ => None,
    }
}

fn get_same(arrs: &[Vec<i32>]) -> Option<i32> {
    let mut inds = vec![0_usize; arrs.len()];
    let mut value = None;

    while let Some(values) = arrs
        .iter()
        .zip(inds.iter())
        .map(|(arr, i)| arr.get(*i).copied())
        .collect::<Option<Vec<i32>>>()
    {
        if values.windows(2).all(|w| w[0] == w[1]) {
            value = values.first().copied();
            break;
        } else {
            if let Some(index_of_min) = values
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.cmp(b))
                .map(|(index, _)| index)
            {
                if let Some(ind) = inds.get_mut(index_of_min) {
                    *ind += 1;
                }
            }
        }
    }

    value
}

fn _find_sum_types(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    input
        .map(|line| try {
            let line: String = line.into();
            if line.len() % 2 != 0 {
                Err(anyhow!(
                    "Number of characters in a line is not divisible by 2"
                ))?
            }
            let halfes: [Vec<i32>; 2] = line
                .chars()
                .collect::<Vec<_>>()
                .chunks(line.len() / 2)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|&c| get_value(c).unwrap())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|e: Vec<_>| anyhow!("wrong number of chunks: {}", e.len()))?;

            get_same(&halfes).ok_or(anyhow!("Missing a singular common type!"))?
        })
        .sum()
}

fn find_sum_badges(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    input
        .map(|line| {
            let line: String = line.into();
            line.chars()
                .map(|c| get_value(c).unwrap())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        })
        .tuples()
        .map(|(ln1, ln2, ln3)| try {
            get_same(&[ln1, ln2, ln3]).ok_or(anyhow!("Missing a singular common type!"))?
        })
        .sum()
}

fn main() {
    let stdin = io::stdin();

    let total = find_sum_badges(stdin.lock().lines().filter_map(|s| s.ok())).unwrap();

    println!("{total}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> String {
        r"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw"
            .to_string()
    }

    #[test]
    fn sum_types_ok() {
        let test_input = test_input();

        let total = _find_sum_types(test_input.lines());

        assert!(total.is_ok());

        assert_eq!(total.unwrap(), 157);
    }

    #[test]
    fn sum_badges_ok() {
        let test_input = test_input();

        let total = find_sum_badges(test_input.lines());

        assert!(total.is_ok());

        assert_eq!(total.unwrap(), 70);
    }
}
