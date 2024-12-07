use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let value1 = solve(input);
    println!("total calibration result: {}", value1);
    let value2 = solve2(input);
    println!("total calibration result v2: {}", value2);
}

fn solve(input: &str) -> i64 {
    let values = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<Operation>, _>>()
        .unwrap();
    let operations = PossibleOperations::new();
    values
        .into_iter()
        .filter(|values| operations.valid_test(&values))
        .map(|values| values.test_value)
        .sum()
}

fn solve2(input: &str) -> i64 {
    let values = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<Operation>, _>>()
        .unwrap();
    let operations = PossibleOperations::new_v2();
    values
        .into_iter()
        .filter(|values| operations.valid_test(&values))
        .map(|values| values.test_value)
        .sum()
}

struct Operation {
    test_value: i64,
    right_hand_values: Vec<i64>,
}

impl FromStr for Operation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let next = parts.next().context("empty test value string")?;
        let test_value = next
            .parse()
            .with_context(|| format!("invalid test value string: {}", next))?;
        let parts = parts.next().context("empty values string")?.trim();
        let values: Vec<i64> = parts
            .split_whitespace()
            .map(|v| v.parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            test_value,
            right_hand_values: values,
        })
    }
}

struct PossibleOperations {
    operations: &'static [fn(left: i64, right: i64) -> i64],
}

impl PossibleOperations {
    pub fn new() -> Self {
        Self {
            operations: &[add, multiply],
        }
    }
    pub fn new_v2() -> Self {
        Self {
            operations: &[add, multiply, concatenation],
        }
    }
    pub fn valid_test(&self, operation: &Operation) -> bool {
        let mut result = Vec::from([operation.right_hand_values.first().copied().unwrap()]);

        for right in operation.right_hand_values.iter().copied().skip(1) {
            result = result
                .into_iter()
                .flat_map(|left| {
                    self.operations
                        .iter()
                        .map(|operation| operation(left, right))
                        .collect::<Vec<_>>()
                })
                .collect();
        }

        result.contains(&operation.test_value)
    }
}

fn add(left: i64, right: i64) -> i64 {
    left + right
}

fn multiply(left: i64, right: i64) -> i64 {
    left * right
}

fn concatenation(left: i64, right: i64) -> i64 {
    10i64.pow((right as f64).log10().floor() as u32 + 1) * left + right
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["190: 10 19
3267: 81 40 27
83: 17 5
156: 15 6
7290: 6 8 6 15
161011: 16 10 13
192: 17 8 14
21037: 9 7 18 13
292: 11 6 16 20"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
