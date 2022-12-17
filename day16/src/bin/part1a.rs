use day16::get_max_pressure;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_max_pressure(stdin.lock().lines().filter_map(|s| s.ok()));

    println!("{}", res.unwrap());

    Ok(())
}
