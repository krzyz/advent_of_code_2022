use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

pub fn start_n(signal: String, n: usize) -> Result<usize> {
    signal
        .chars()
        .collect::<Vec<_>>()
        .windows(n)
        .enumerate()
        .find_map(|(i, window)| (window.iter().collect::<BTreeSet<_>>().len() == n).then(|| i + n))
        .ok_or(anyhow!("not found"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("bvwbjplbgvbhsrlpgdmjqwftvncz", 4, 5)]
    #[case("nppdvjthqldpwncqszvftbrmjlhg", 4, 6)]
    #[case("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 4, 10)]
    #[case("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 4, 11)]
    #[case("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 14, 19)]
    #[case("bvwbjplbgvbhsrlpgdmjqwftvncz", 14, 23)]
    #[case("nppdvjthqldpwncqszvftbrmjlhg", 14, 23)]
    #[case("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 14, 29)]
    #[case("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 14, 26)]
    fn part1(#[case] input: String, #[case] n: usize, #[case] expected: usize) {
        let res = start_n(input, n);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), expected);
    }
}
