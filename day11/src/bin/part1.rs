#![feature(iter_intersperse)]

use day11::get_monkey_business;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_monkey_business(
        stdin
            .lock()
            .lines()
            .intersperse_with(|| Ok("\n".to_string()))
            .collect::<std::result::Result<String, _>>()
            .map_err(anyhow::Error::msg)?
            .as_str(),
        20,
    );

    println!("{}", res.unwrap());

    Ok(())
}
