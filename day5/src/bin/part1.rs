use anyhow::Result;
use day5::move_and_get_top_9000;
use std::io::{self, BufRead};

fn main() -> Result<()> {
    let stdin = io::stdin();

    let total = move_and_get_top_9000(stdin.lock().lines().filter_map(|s| s.ok()))?;

    println!("{total}");

    Ok(())
}
