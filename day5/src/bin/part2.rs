use anyhow::Result;
use day5::move_and_get_top_9001;
use std::io::{self, BufRead};

fn main() -> Result<()> {
    let stdin = io::stdin();

    let total = move_and_get_top_9001(stdin.lock().lines().filter_map(|s| s.ok()))?;

    println!("{total}");

    Ok(())
}
