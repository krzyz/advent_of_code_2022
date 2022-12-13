use day13::get_decoder_key;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_decoder_key(stdin.lock().lines().filter_map(|s| s.ok()));

    println!("{}", res.unwrap());

    Ok(())
}
