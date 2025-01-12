use crate::common::{Context, InputProvider};
use std::fmt::{Display, Formatter};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let (locks, keys) = parse_locks_and_keys(input);

    let mut fit = 0;
    for (lock, key) in locks
        .iter()
        .flat_map(|lock| keys.iter().map(move |key| (lock, key)))
    {
        if let Some(overlap) = key.key_overlaps_lock(lock) {
            log::debug!(
                "Lock {} and key {} overlap in column {}",
                lock,
                key,
                overlap
            );
        } else {
            log::debug!("Lock {} and key {} fit!", lock, key);
            fit += 1;
        }
    }
    println!("unique lock/key pairs that fit together: {}", fit);
}

fn parse_locks_and_keys(input: &str) -> (Vec<LockOrKey>, Vec<LockOrKey>) {
    let mut locks = Vec::new();
    let mut keys = Vec::new();
    for input in input.split("\n\n") {
        let mut counts = [0u32; 5];
        for line in input.lines() {
            for (i, char) in line.chars().enumerate() {
                match char {
                    '.' => {}
                    '#' => {
                        counts[i] += 1;
                    }
                    other => panic!("invalid char '{}'", other),
                }
            }
        }
        for count in counts.iter_mut() {
            if *count == 0 || *count > 6 {
                panic!("invalid count {}", count);
            }
            *count -= 1;
        }
        match input.lines().next() {
            None => continue,
            Some(".....") => keys.push(LockOrKey(counts)),
            Some("#####") => locks.push(LockOrKey(counts)),
            Some(other) => panic!("invalid start of lock or key: '{}'", other),
        }
    }

    (locks, keys)
}

struct LockOrKey([u32; 5]);

impl LockOrKey {
    pub fn key_overlaps_lock(&self, lock: &Self) -> Option<usize> {
        for (i, (key, lock)) in self.0.iter().zip(lock.0.iter()).enumerate() {
            if key + lock > 5 {
                return Some(i);
            }
        }
        None
    }
}

impl Display for LockOrKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.map(|v| v.to_string()).join(","))
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["#####
.####
.####
.####
.#.#.
.#...
.....

#####
##.##
.#.##
...##
...#.
...#.
.....

.....
#....
#....
#...#
#.#.#
#.###
#####

.....
.....
#.#..
###..
###.#
###.#
#####

.....
.....
.....
#....
#.#..
#.#.#
#####"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
