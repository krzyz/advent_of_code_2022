use day10::get_crt_output;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let output = get_crt_output(stdin.lock().lines().filter_map(|s| s.ok()), 6, 40);

    println!("{}", output);

    Ok(())
}
