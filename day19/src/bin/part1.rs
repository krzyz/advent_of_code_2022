use day19::get_sum_quality_levels;
use util::read_input_as_string;

use anyhow::Result;

fn main() -> Result<()> {
    let res = get_sum_quality_levels(read_input_as_string()?.as_str());

    println!("{}", res.unwrap());

    Ok(())
}
