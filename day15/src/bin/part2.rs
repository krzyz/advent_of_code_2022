use day15::get_distress_beacon_freq;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_distress_beacon_freq(stdin.lock().lines().filter_map(|s| s.ok()), 0, 4000000);

    println!("{}", res.unwrap());

    Ok(())
}
