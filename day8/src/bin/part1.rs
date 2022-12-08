use day8::get_num_visible;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_num_visible(stdin.lock().lines().filter_map(|s| s.ok()));

    println!("{}", res.unwrap());

    Ok(())
}
