use day17::get_tower_height;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_tower_height(stdin.lock().lines().filter_map(|s| s.ok()), 2022)?;

    println!("{res}");

    Ok(())
}
