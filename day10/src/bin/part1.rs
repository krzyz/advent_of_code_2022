use day10::get_sum_signal_strengths;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_sum_signal_strengths(stdin.lock().lines().filter_map(|s| s.ok()), 20, 40, 6);

    println!("{}", res.unwrap());

    Ok(())
}
