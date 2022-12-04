#![feature(try_blocks)]

use anyhow::{anyhow, Result};
use std::io::{self, BufRead};

fn num_overlap(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    input
        .map(|line| try {
            let line: String = line.into();

            let ranges: [i32; 4] = line
                .split([',', '-'])
                .map(|n| n.parse::<i32>().map_err(|e| anyhow!(e.to_string())))
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .map_err(|e: Vec<_>| {
                    anyhow!("wrong number of range starts/ends in a line: {}", e.len())
                })?;

            let first_constains_second = ranges[0] <= ranges[2] && ranges[1] >= ranges[3];
            let second_constains_first = ranges[2] <= ranges[0] && ranges[3] >= ranges[1];

            if first_constains_second || second_constains_first {
                1
            } else {
                0
            }
        })
        .sum()
}

fn main() {
    let stdin = io::stdin();

    let total = num_overlap(stdin.lock().lines().filter_map(|s| s.ok())).unwrap();

    println!("{total}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> String {
        r"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8"
            .to_string()
    }

    #[test]
    fn num_overlap_ok() {
        let test_input = test_input();

        let total = num_overlap(test_input.lines());

        assert!(total.is_ok());

        assert_eq!(total.unwrap(), 2);
    }
}
