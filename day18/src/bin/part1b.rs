use day18::get_num_exposed_sides_2;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_num_exposed_sides_2(stdin.lock().lines().filter_map(|s| s.ok()), false)?;

    println!("{res}");

    Ok(())
}
