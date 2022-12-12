#![feature(iter_intersperse)]

use day12::get_shortest_path_len;

use std::io::{self, BufRead};

use anyhow::Result;

fn main() -> Result<()> {
    let stdin = io::stdin();

    let res = get_shortest_path_len(
        stdin
            .lock()
            .lines()
            .intersperse_with(|| Ok("\n".to_string()))
            .collect::<std::result::Result<String, _>>()
            .map_err(anyhow::Error::msg)?
            .as_str(),
    );

    println!("{}", res.unwrap());

    Ok(())
}
