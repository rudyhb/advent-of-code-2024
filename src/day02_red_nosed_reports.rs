use crate::common::{Context, InputProvider};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let safe_count = input
        .lines()
        .filter(|&line| line.parse::<Report>().unwrap().is_safe())
        .count();
    println!("Safe reports: {}", safe_count);

    let safe_count = input
        .lines()
        .filter(|&line| line.parse::<Report>().unwrap().is_safe_v2())
        .count();
    println!("Safe reports v2: {}", safe_count);
}

struct Report(Vec<i32>);

impl Report {
    pub fn is_safe(&self) -> bool {
        Self::levels_are_safe(&self.0)
    }
    fn levels_are_safe(numbers: &[i32]) -> bool {
        let (min, max) = if numbers[1] > numbers[0] {
            (1, 3)
        } else {
            (-3, -1)
        };

        numbers.windows(2).all(|window| {
            let &[left, right] = window else {
                unreachable!()
            };
            let diff = right - left;
            diff >= min && diff <= max
        })
    }
    pub fn is_safe_v2(&self) -> bool {
        let numbers = &self.0;

        if Self::levels_are_safe(&numbers[1..]) {
            log::debug!("safe without 0");
            return true;
        }

        for i in 1..numbers.len() {
            let mut numbers = numbers.clone();
            numbers.remove(i);
            if Self::levels_are_safe(&numbers) {
                log::debug!("safe without {}", i);
                return true;
            }
        }

        log::debug!("unsafe");
        false
    }
}

impl FromStr for Report {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_whitespace()
                .map(|c| c.parse())
                .collect::<Result<Vec<i32>, _>>()?,
        ))
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
