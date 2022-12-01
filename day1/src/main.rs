use std::{
    collections::BTreeSet,
    io::{self, BufRead},
};

fn find_ordered_totals(input: impl Iterator<Item = impl Into<String>>) -> BTreeSet<i32> {
    let mut totals = BTreeSet::new();
    let mut current_total = 0;
    for line in input {
        let line: String = line.into();
        if line.is_empty() {
            totals.insert(current_total);
            current_total = 0;
        } else {
            let calories = line.parse::<i32>().unwrap();
            current_total += calories;
        }
    }

    totals
}

fn get_biggest_three_total(set: &BTreeSet<i32>) -> i32 {
    set.iter().rev().take(3).sum()
}

fn main() {
    let stdin = io::stdin();

    let totals = find_ordered_totals(stdin.lock().lines().filter_map(|s| s.ok()));
    let answer = get_biggest_three_total(&totals);

    println!("{answer}");
}
