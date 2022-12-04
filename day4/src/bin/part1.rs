use anyhow::Result;
use day4::num_overlap_full;
use std::io::{self, BufRead};

fn main() -> Result<()> {
    let stdin = io::stdin();

    let total = num_overlap_full(stdin.lock().lines().filter_map(|s| s.ok()))?;

    println!("{total}");

    Ok(())
}
