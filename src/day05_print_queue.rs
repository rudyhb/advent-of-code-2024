use crate::common::{Context, InputProvider};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let input_parts = input.split_once("\n\n").unwrap();
    let rules = OrderRules::from_str(input_parts.0).unwrap();

    let (sum, sum2) = solve(&rules, input_parts.1);
    println!("the sum of the middle pages is {}", sum);
    println!("the sum of the corrected middle pages is {}", sum2);
}

fn solve(rules: &OrderRules, input: &str) -> (u32, u32) {
    let mut sum = 0;
    let mut sum2 = 0;

    fn fix_and_add_sum(rules: &OrderRules, values: &[u32], sum: &mut u32) {
        let mut sorted: Vec<u32> = Vec::new();
        let mut values_left: Vec<_> = values.to_vec();

        while !values_left.is_empty() {
            let cannot_be_chosen: HashSet<u32> = values_left
                .iter()
                .filter_map(|left| rules.rules.get(left))
                .flatten()
                .copied()
                .collect();
            let (index, next) = values_left
                .iter()
                .enumerate()
                .filter(|(_, left)| !cannot_be_chosen.contains(left))
                .next()
                .unwrap();
            sorted.push(*next);
            values_left.remove(index);
        }

        let middle = sorted.into_iter().nth(values.len() / 2).unwrap_or_default();
        *sum += middle;
    }

    'outer: for line in input.lines() {
        let values = line
            .split(',')
            .map(|s| s.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let mut existing: HashSet<u32> = HashSet::new();
        for value in values.iter() {
            if let Some(not_before) = rules.rules.get(value) {
                if not_before.iter().any(|right| existing.contains(right)) {
                    fix_and_add_sum(rules, &values, &mut sum2);
                    continue 'outer;
                }
            }
            existing.insert(*value);
        }
        sum += values
            .iter()
            .nth(values.len() / 2)
            .copied()
            .unwrap_or_default();
    }

    (sum, sum2)
}

struct OrderRules {
    rules: HashMap<u32, Vec<u32>>,
}

impl FromStr for OrderRules {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rules = s
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('|');
                if let Some(left) = parts.next() {
                    if let Ok(left) = left.parse::<u32>() {
                        let right = parts.next().unwrap().parse::<u32>().unwrap();
                        return Some((left, right));
                    }
                }
                None
            })
            .fold(HashMap::new(), |mut acc, (left, right)| {
                acc.entry(left).or_insert_with(Vec::new).push(right);
                acc
            });
        Ok(OrderRules { rules })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
