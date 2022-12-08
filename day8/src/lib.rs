#![feature(try_blocks)]

use std::{cmp::Ordering, collections::HashMap};

use anyhow::{anyhow, Context, Result};
use num::range_step_inclusive;

fn get_trees(input: impl Iterator<Item = String>) -> Result<Vec<Vec<usize>>> {
    let trees = input
        .map(|l| try {
            let usize_vec = l
                .chars()
                .map(|c| c.to_string().parse().map_err(anyhow::Error::from))
                .collect::<Result<Vec<usize>>>()?;

            usize_vec
        })
        .collect::<Result<Vec<Vec<usize>>>>()?;

    Ok(trees)
}

fn get_n_with_check(matrix: &Vec<Vec<usize>>) -> Result<usize> {
    let n_rows = matrix.len();
    matrix
        .iter()
        .all(|l| l.len() == n_rows)
        .then_some(())
        .ok_or(anyhow!(
            "Not all rows have the same length as column length!"
        ))?;
    Ok(n_rows)
}

pub fn run_all_directions<F>(trees: &Vec<Vec<usize>>, mut func: F) -> Result<()>
where
    F: FnMut((usize, usize, usize, bool)),
{
    let n = trees.len();

    enum OuterI {
        Rows,
        Cols,
    }

    let range_max = n.checked_sub(1).context("no values in the matrix!")? as i32;

    let specs = vec![
        ((0, range_max, 1i32), OuterI::Rows),
        ((range_max, 0, -1), OuterI::Rows),
        ((0, range_max, 1), OuterI::Cols),
        ((range_max, 0, -1), OuterI::Cols),
    ];

    for ((inner_start, inner_stop, inner_step), outer_which) in specs {
        for outer in 0..n {
            let mut new_outer = true;
            for inner in range_step_inclusive(inner_start, inner_stop, inner_step) {
                let inner = inner as usize;
                let (row, col) = match outer_which {
                    OuterI::Rows => (outer, inner),
                    OuterI::Cols => (inner, outer),
                };
                let val = trees.get(row).unwrap().get(col).unwrap();
                func((row, col, *val, new_outer));
                new_outer = false;
            }
        }
    }

    Ok(())
}

fn get_scenic_scores(input: impl Iterator<Item = String>) -> Result<Vec<Vec<usize>>> {
    let trees = get_trees(input)?;
    let n = get_n_with_check(&trees)?;

    let mut scenic_scores = vec![vec!(1; n); n];

    let mut current_scores = HashMap::new();

    run_all_directions(&trees, |(row, col, val, new_outer)| {
        if new_outer {
            current_scores.clear();
        }

        let cumulative_score = scenic_scores.get_mut(row).unwrap().get_mut(col).unwrap();
        let current_score = current_scores.get(&val).copied().unwrap_or(0);
        *cumulative_score *= current_score;

        for d in 0..=9 {
            match val.cmp(&d) {
                Ordering::Less => {
                    *current_scores.entry(d).or_insert(0) += 1;
                }
                Ordering::Equal | Ordering::Greater => {
                    current_scores.entry(d).and_modify(|v| *v = 1).or_insert(1);
                }
            }
        }
    })?;

    Ok(scenic_scores)
}

pub fn get_greatest_scenic_score(input: impl Iterator<Item = String>) -> Result<usize> {
    get_scenic_scores(input)?
        .iter()
        .flatten()
        .max()
        .copied()
        .context("No scenic scores!")
}

pub fn get_num_visible(input: impl Iterator<Item = String>) -> Result<usize> {
    let trees = get_trees(input)?;

    let n = get_n_with_check(&trees)?;

    let mut visible = vec![vec!(false; n); n];
    let mut current_biggest = None;

    run_all_directions(&trees, |(row, col, val, new_outer)| {
        if new_outer {
            current_biggest = None;
        }

        let visible_val = visible.get_mut(row).unwrap().get_mut(col).unwrap();

        let new_biggest = if let Some(current_biggest) = current_biggest {
            if val > current_biggest {
                Some(val)
            } else {
                None
            }
        } else {
            Some(val)
        };

        if let Some(new_biggest) = new_biggest {
            current_biggest = Some(new_biggest);
            *visible_val = true;
        }
    })
    .map_err(anyhow::Error::msg)?;

    Ok(visible
        .iter()
        .flatten()
        .filter_map(|v| v.then_some(1))
        .sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = include_str!("../data/test_input");

    #[test]
    fn part1() {
        let res = get_num_visible(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 21);
    }

    #[test]
    fn part2() {
        let res = get_greatest_scenic_score(TEST_INPUT.lines().map(|l| l.to_string()));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 8);
    }
}
