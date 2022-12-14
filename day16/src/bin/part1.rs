use day16::get_max_pressure_2;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_max_pressure_2(stdin.lock().lines().filter_map(|s| s.ok()), 1, 30);

    println!("{}", res.unwrap());

    Ok(())
}
