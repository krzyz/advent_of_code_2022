use day13::get_sum_right_order;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_sum_right_order(stdin.lock().lines().filter_map(|s| s.ok()));

    println!("{}", res.unwrap());

    Ok(())
}
