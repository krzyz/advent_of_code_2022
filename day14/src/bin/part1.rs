use day14::get_num_sand_rest;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_num_sand_rest(stdin.lock().lines().filter_map(|s| s.ok()), false, false);

    println!("{}", res.unwrap());

    Ok(())
}
