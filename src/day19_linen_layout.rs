use crate::common::{Context, InputProvider};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::rc::Rc;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let mut parts = input.split("\n\n");
    let available_patterns: AvailablePatterns = parts.next().unwrap().parse().unwrap();
    let target_patterns: Vec<Pattern> = parts
        .next()
        .unwrap()
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let possible_count: usize = target_patterns
        .iter()
        .map(|target| {
            let count = available_patterns
                .filtered_by_size(&target.0)
                .can_create_count_memo(&target.0);
            println!("{:?} can be made {} ways", target.0, count);
            count
        })
        .sum();
    println!("{} ways to do designs", possible_count);
}

struct AvailablePatterns {
    available: Vec<Pattern>,
}

impl AvailablePatterns {
    pub fn filtered_by_size(&self, target: &[char]) -> AvailablePatternsBySize {
        let target: String = target.iter().collect();

        let result = AvailablePatternsBySize {
            patterns: self
                .available
                .iter()
                .filter(|pattern| {
                    let pattern: String = pattern.0.iter().collect();
                    target.contains(&pattern)
                })
                .map(|pattern| (pattern.0.len(), pattern.clone()))
                .fold(HashMap::new(), |mut acc, (len, pattern)| {
                    acc.entry(len).or_default().push(pattern);
                    acc
                }),
            cache: Rc::new(RefCell::new(Default::default())),
        };
        log::debug!(
            "filtered {} -> {}:\n{:?}",
            self.available.len(),
            result.patterns.iter().map(|p| p.1.len()).sum::<usize>(),
            result
                .patterns
                .iter()
                .map(|p| format!("{}x len{}", p.1.len(), p.0))
                .collect::<Vec<_>>()
        );
        result
    }
}

struct AvailablePatternsBySize {
    patterns: HashMap<usize, Vec<Pattern>>,
    cache: Rc<RefCell<HashMap<u64, usize>>>,
}

impl AvailablePatternsBySize {
    pub fn can_create_count_memo(&self, target: &[char]) -> usize {
        if let Some(cached) = self.get_cache(target) {
            return cached;
        }
        let result = self
            .patterns
            .iter()
            .filter(|(&len, _)| target.len() >= len)
            .map(|(&len, patterns)| {
                let count = patterns
                    .iter()
                    .filter(|pattern| pattern.0 == target[0..len])
                    .count();
                if target.len() == len {
                    count
                } else {
                    count * self.can_create_count_memo(&target[len..])
                }
            })
            .sum();

        self.set_cache(target, result);
        result
    }
    pub fn get_cache(&self, target: &[char]) -> Option<usize> {
        self.cache.borrow().get(&hash(target)).copied()
    }
    pub fn set_cache(&self, target: &[char], result: usize) {
        self.cache.borrow_mut().insert(hash(target), result);
    }
}

fn hash(target: &[char]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for &c in target {
        hasher.write_u8(c as u8);
    }
    hasher.finish()
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct Pattern(Vec<char>);

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.trim().chars().collect()))
    }
}

impl FromStr for AvailablePatterns {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            available: s
                .split(',')
                .map(|v| v.parse())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["r, wr, b, g, bwu, rb, gb, br

brwrr
bggr
gbbr
rrbgbr
ubwu
bwurrg
brgr
bbrgwb
"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
