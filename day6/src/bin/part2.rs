use day6::start_n;

use std::io::{self, BufRead};

use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    let stdin = io::stdin();

    let n = start_n(
        stdin
            .lock()
            .lines()
            .filter_map(|s| s.ok())
            .next()
            .ok_or(anyhow!("No line in input"))?,
        14,
    )?;

    println!("{n}");

    Ok(())
}
