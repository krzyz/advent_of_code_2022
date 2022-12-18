use day18::get_num_exposed_sides;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_num_exposed_sides(stdin.lock().lines().filter_map(|s| s.ok()))?;

    println!("{res}");

    Ok(())
}
