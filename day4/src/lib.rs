#![feature(try_blocks)]

use anyhow::{anyhow, Result};

fn num_overlap_condition<F>(
    input: impl Iterator<Item = impl Into<String>>,
    condition_true: F,
) -> Result<i32>
where
    F: Fn(&[i32; 4]) -> bool,
{
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

            if condition_true(&ranges) {
                1
            } else {
                0
            }
        })
        .sum()
}

fn condition_full_overlap(ranges: &[i32; 4]) -> bool {
    let first_constains_second = ranges[0] <= ranges[2] && ranges[1] >= ranges[3];
    let second_constains_first = ranges[2] <= ranges[0] && ranges[3] >= ranges[1];

    first_constains_second || second_constains_first
}

fn condition_any_overlap(ranges: &[i32; 4]) -> bool {
    !(ranges[0] > ranges[3] || ranges[1] < ranges[2])
}

pub fn num_overlap_full(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    num_overlap_condition(input, condition_full_overlap)
}

pub fn num_overlap_any(input: impl Iterator<Item = impl Into<String>>) -> Result<i32> {
    num_overlap_condition(input, condition_any_overlap)
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
    fn num_overlap_full_ok() {
        let test_input = test_input();

        let total = num_overlap_full(test_input.lines());

        assert!(total.is_ok());

        assert_eq!(total.unwrap(), 2);
    }

    #[test]
    fn num_overlap_any_ok() {
        let test_input = test_input();

        let total = num_overlap_any(test_input.lines());

        assert!(total.is_ok());

        assert_eq!(total.unwrap(), 4);
    }
}
