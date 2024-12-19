use crate::common::{Context, InputProvider};
use std::collections::HashMap;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let (mut left, mut right) = parse(context.get_input().as_str());
    left.sort();
    right.sort();
    let distance = get_distance(&left, &right);

    println!("Distance: {}", distance);

    let similarity_score = get_similarity_score(&left, &right);
    println!("Similarity score: {}", similarity_score);
}

fn parse(input: &str) -> (Vec<i32>, Vec<i32>) {
    let i = input.lines().count();
    let mut left: Vec<i32> = Vec::with_capacity(i);
    let mut right: Vec<i32> = Vec::with_capacity(i);
    for line in input.lines() {
        log::debug!("line: {}", line);
        let mut nums = line.split_whitespace().map(|n| n.parse().unwrap());
        left.push(nums.next().unwrap());
        right.push(nums.next().unwrap());
    }
    (left, right)
}

fn get_distance(left: &[i32], right: &[i32]) -> i32 {
    left.into_iter()
        .copied()
        .zip(right.into_iter().copied())
        .map(|(l, r)| {
            log::debug!("{} - {} = {}", l, r, (l - r).abs());
            (l - r).abs()
        })
        .sum()
}

fn get_similarity_score(left: &[i32], right: &[i32]) -> i32 {
    let right = right
        .into_iter()
        .copied()
        .fold(HashMap::new(), |mut map, r| {
            *map.entry(r).or_insert(0) += 1;
            map
        });
    left.into_iter()
        .copied()
        .map(|l| {
            let count = right.get(&l).unwrap_or(&0);
            log::debug!("{}: {}", l, count);
            *count * l
        })
        .sum()
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["3   4
    4   3
    2   5
    1   3
    3   9
    3   3"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
