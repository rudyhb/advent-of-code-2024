use crate::common::{Context, InputProvider};
use std::collections::{HashMap, LinkedList};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let mut stones: Stones = input.parse().unwrap();

    log::debug!("initial:\n{}", stones);

    for i in 1..=25 {
        stones.blink();

        if i < 7 {
            log::debug!("after {} blinks:\n{}", i, stones);
        }
    }

    println!("stones after 25 times: {}", stones.stones_count());

    let mut stones: StonesV2 = input.parse::<Stones>().unwrap().into();

    for _ in 1..=75 {
        stones.blink();
    }

    println!("stones after 75 times: {}", stones.stones_count());
}

struct Stones(LinkedList<Stone>);

impl Stones {
    pub fn stones_count(&self) -> usize {
        self.0.len()
    }
    pub fn blink(&mut self) {
        let mut cursor = self.0.cursor_front_mut();

        while let Some(stone) = cursor.current() {
            if let Some(split_right) = stone.blink() {
                cursor.insert_after(split_right);
                cursor.move_next();
            }
            cursor.move_next();
        }
    }
}

impl FromStr for Stones {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_whitespace()
                .map(|value| value.parse().map(|val| Stone::new(val)))
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl Display for Stones {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for val in &self.0 {
            write!(f, "{}  ", val.0)?;
        }
        Ok(())
    }
}

struct StonesV2(HashMap<Stone, usize>);

impl From<Stones> for StonesV2 {
    fn from(value: Stones) -> Self {
        let mut map = HashMap::new();
        for stone in value.0 {
            *map.entry(stone).or_default() += 1;
        }
        Self(map)
    }
}

impl StonesV2 {
    pub fn stones_count(&self) -> usize {
        self.0.iter().map(|(_, &count)| count).sum()
    }
    pub fn blink(&mut self) {
        let mut result = HashMap::new();
        for (mut stone, count) in std::mem::take(&mut self.0) {
            if let Some(split) = stone.blink() {
                *result.entry(split).or_default() += count;
            }
            *result.entry(stone).or_default() += count;
        }

        self.0 = result;
    }
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Stone(u64);

impl Stone {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    pub fn blink(&mut self) -> Option<Self> {
        if self.0 == 0 {
            self.0 = 1;
            return None;
        }

        let digits = 1 + self.0.ilog10();

        if digits % 2 == 0 {
            let tens = 10u64.pow(digits / 2);
            let left = self.0 / tens;
            let right = self.0 % tens;

            self.0 = left;
            return Some(Self::new(right));
        }

        self.0 *= 2024;
        None
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["125 17"]
        .into_iter()
        .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
