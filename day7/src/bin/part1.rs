use day7::size_smallest;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let cs = size_smallest(stdin.lock().lines().filter_map(|s| s.ok()), 100000);

    println!("{cs}");

    Ok(())
}
