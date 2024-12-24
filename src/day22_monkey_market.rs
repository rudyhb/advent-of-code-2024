use crate::common::{Context, InputProvider};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let mut secret_numbers: Vec<SecretNumber> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut aggregators: Vec<FourStepAggregator> = vec![Default::default(); secret_numbers.len()];

    secret_numbers
        .iter_mut()
        .enumerate()
        .for_each(|(i, secret)| {
            for _ in 0..2000 {
                secret.next();
                if let Some(change) = secret.get_diffs() {
                    aggregators[i].insert_once(change, secret.get_price());
                }
            }
        });
    let sum: i64 = secret_numbers.iter().map(|n| n.secret).sum();
    println!("sum of 2000th secret number: {}", sum);

    let all_changes: HashMap<FourStepChange, i64> = aggregators
        .iter()
        .flat_map(|aggregator| {
            aggregator
                .changes_to_price
                .iter()
                .map(|(change, bananas)| (change.clone(), *bananas))
        })
        .fold(HashMap::new(), |mut acc, (change, bananas)| {
            *acc.entry(change).or_default() += bananas;
            acc
        });

    let (max_change, &max) = all_changes
        .iter()
        .max_by(|(_, a), (_, b)| a.cmp(b))
        .unwrap();

    println!("max bananas: {:?} {}", max_change.0, max);
}

#[derive(Clone, Default)]
struct FourStepAggregator {
    changes_to_price: HashMap<FourStepChange, i64>,
}

impl FourStepAggregator {
    pub fn insert_once(&mut self, change: FourStepChange, price: i64) {
        self.changes_to_price.entry(change).or_insert(price);
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct FourStepChange([i64; 4]);

impl FourStepChange {
    pub fn new(values: &[i64]) -> Self {
        Self(values.try_into().unwrap())
    }
}

struct SecretNumber {
    secret: i64,
    diffs: Vec<i64>,
    last: i64,
}

impl SecretNumber {
    const MODULO: i64 = 16777216;

    pub fn new(value: i64) -> Self {
        Self {
            secret: value,
            diffs: vec![],
            last: 0,
        }
    }

    pub fn get_price(&self) -> i64 {
        self.secret % 10
    }

    pub fn get_diffs(&self) -> Option<FourStepChange> {
        if self.diffs.len() >= 4 {
            Some(FourStepChange::new(&self.diffs[self.diffs.len() - 4..]))
        } else {
            None
        }
    }

    pub fn next(&mut self) {
        self.last = self.secret;
        self.multiply_64();
        self.prune();
        self.divide_32();
        self.prune();
        self.multiply_2048();
        self.prune();
        self.diffs.push((self.secret % 10) - (self.last % 10));
    }
    fn multiply_64(&mut self) {
        self.secret ^= self.secret * 64;
    }
    fn divide_32(&mut self) {
        self.secret ^= self.secret / 32;
    }
    fn multiply_2048(&mut self) {
        self.secret ^= self.secret * 2048;
    }
    fn prune(&mut self) {
        self.secret %= Self::MODULO;
    }
}

impl FromStr for SecretNumber {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

impl Display for SecretNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.secret)
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "1
2
3
2024",
        "1
10
100
2024",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
