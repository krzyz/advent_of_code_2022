use day7::size_to_delete;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let cs = size_to_delete(
        stdin.lock().lines().filter_map(|s| s.ok()),
        70000000,
        30000000,
    );

    println!("{cs}");

    Ok(())
}
