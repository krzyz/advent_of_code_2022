use day8::get_greatest_scenic_score;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_greatest_scenic_score(stdin.lock().lines().filter_map(|s| s.ok()));

    println!("{}", res.unwrap());

    Ok(())
}
