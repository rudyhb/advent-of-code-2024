use crate::common::models::{Direction, Point};
use crate::common::{Context, InputProvider};
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let door_codes: Vec<NumericSequence> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut solver = Solver::default();
    let sum_complexities: usize = door_codes
        .clone()
        .into_iter()
        .map(|code| {
            let best = solver.solve_door(&code, 3);
            let numeric_code = code.numeric_code();
            log::debug!("shortest code for {}:\n{}", code, best);
            log::debug!(
                "complexity: length {} * numeric {} = {}",
                best.0.len(),
                numeric_code,
                best.0.len() * numeric_code
            );
            best.0.len() * numeric_code
        })
        .sum();
    println!("sum complexities 3 robots: {}", sum_complexities);

    let robots = 26;
    let sum_complexities: usize = door_codes
        .into_iter()
        .map(|code| {
            let best = solve_door(&code, robots);
            let numeric_code = code.numeric_code();
            log::debug!("shortest code for {}:\n{}", code, best);
            log::debug!(
                "complexity: length {} * numeric {} = {}",
                best,
                numeric_code,
                best * numeric_code
            );
            best * numeric_code
        })
        .sum();

    println!("sum complexities {} robots: {}", robots, sum_complexities);
}
const PEEK_DISTANCE: usize = 5;
fn find_next(
    codes: DirectionalSequence,
    count: usize,
    cache: &mut HashMap<DirectionalSequence, DirectionalSequence>,
) -> DirectionalSequence {
    if let Some(result) = cache.get(&codes) {
        return result.clone();
    }
    let result = find_next_iter(codes.clone(), count).1;
    cache.insert(codes, result.clone());

    result
}
fn find_next_iter(codes: DirectionalSequence, count: usize) -> (usize, DirectionalSequence) {
    if count == 0 {
        return (codes.0.len(), codes);
    }
    let mut active = Value::Directional(Default::default());
    let (len, next) = codes
        .0
        .iter()
        .map(|&value| {
            let diffs =
                get_direction_diffs_for_directional_keypad(Value::Directional(value), active);
            active = Value::Directional(value);
            let (len, next) = diffs
                .iter()
                .map(|directions| {
                    let next = DirectionalSequence::from_directions(&directions);
                    let len = find_next_iter(next.clone(), count - 1).0;
                    (len, next)
                })
                .min_by(|(a, _), (b, _)| a.cmp(b))
                .unwrap();
            (len, next)
        })
        .fold(
            (0, DirectionalSequence::default()),
            |(mut sum, mut acc), (len, next)| {
                sum += len;
                acc.0.extend(next.0);
                (sum, acc)
            },
        );

    (len, next)
}
#[derive(Eq, PartialEq, Hash, Clone)]
struct SolveCacheIndex<'a> {
    sequence: Cow<'a, DirectionalSequence>,
    robot_count: usize,
}
impl<'a> SolveCacheIndex<'a> {
    pub fn new(sequence: Cow<'a, DirectionalSequence>, robot_count: usize) -> Self {
        Self {
            sequence,
            robot_count,
        }
    }
}
fn solve_keypad(
    directions: DirectionalSequence,
    robot_count: usize,
    peek_cache: &mut HashMap<DirectionalSequence, DirectionalSequence>,
    solve_cache: &mut HashMap<SolveCacheIndex, usize>,
) -> usize {
    if robot_count == 0 {
        return directions.0.len();
    }

    let index = SolveCacheIndex::new(Cow::Borrowed(&directions), robot_count);
    if let Some(&solution) = solve_cache.get(&index) {
        return solution;
    }

    let solution = directions
        .iter_actions()
        .map(|action| {
            let next = find_next(action, PEEK_DISTANCE, peek_cache);
            solve_keypad(next, robot_count - 1, peek_cache, solve_cache)
        })
        .sum();

    solve_cache.insert(
        SolveCacheIndex::new(Cow::Owned(directions), robot_count),
        solution,
    );

    solution
}
fn solve_door(code: &NumericSequence, robot_count: usize) -> usize {
    let mut active = Value::Numeric(Default::default());
    let mut cache = HashMap::default();
    let mut cache2 = HashMap::default();
    let result = code
        .0
        .iter()
        .map(|&value| {
            let diffs = get_direction_diffs_for_numeric_keypad(Value::Numeric(value), active);
            active = Value::Numeric(value);
            diffs
                .iter()
                .map(|directions| {
                    solve_keypad(
                        DirectionalSequence::from_directions(&directions),
                        robot_count - 1,
                        &mut cache,
                        &mut cache2,
                    )
                })
                .min()
                .unwrap()
        })
        .sum();
    log::debug!(
        "caches 1: count={}, length frequencies={:?}",
        cache.len(),
        cache
            .iter()
            .fold(HashMap::new(), |mut acc: HashMap<usize, usize>, next| {
                *acc.entry(next.0.len()).or_default() += 1;
                acc
            })
    );
    log::debug!("caches 2: count={}", cache2.len());
    result
}

#[derive(Default)]
struct Solver {
    cache_next: HashMap<DirectionalSequence, DirectionalSequence>,
}

impl Solver {
    const PEEK_DISTANCE: usize = 5;
    fn find_next(&mut self, codes: DirectionalSequence, count: usize) -> DirectionalSequence {
        if let Some(cache) = self.cache_next.get(&codes) {
            return cache.clone();
        }
        let result = Self::find_next_iter(codes.clone(), count).1;
        self.cache_next.insert(codes, result.clone());
        result
    }
    fn find_next_iter(codes: DirectionalSequence, count: usize) -> (usize, DirectionalSequence) {
        if count == 0 {
            return (codes.0.len(), codes);
        }
        let mut active = Value::Directional(Default::default());
        let (len, next) = codes
            .0
            .iter()
            .map(|&value| {
                let diffs =
                    get_direction_diffs_for_directional_keypad(Value::Directional(value), active);
                active = Value::Directional(value);
                let (len, next) = diffs
                    .iter()
                    .map(|directions| {
                        let next = DirectionalSequence::from_directions(&directions);
                        let len = Self::find_next_iter(next.clone(), count - 1).0;
                        (len, next)
                    })
                    .min_by(|(a, _), (b, _)| a.cmp(b))
                    .unwrap();
                (len, next)
            })
            .fold(
                (0, DirectionalSequence::default()),
                |(mut sum, mut acc), (len, next)| {
                    sum += len;
                    acc.0.extend(next.0);
                    (sum, acc)
                },
            );

        (len, next)
    }
    fn get_shortest_directions_to_press(
        &mut self,
        directions: DirectionalSequence,
        robot_count: usize,
    ) -> DirectionalSequence {
        let mut directions = directions;
        for i in 0..robot_count {
            log::trace!("robot {} types {}", i, directions);
            directions = directions
                .iter_actions()
                .map(|action| self.find_next(action, Self::PEEK_DISTANCE))
                .fold(DirectionalSequence::default(), |mut acc, next| {
                    acc.0.extend(next.0);
                    acc
                });
        }
        directions
    }
    pub fn solve_door(
        &mut self,
        code: &NumericSequence,
        robot_count: usize,
    ) -> DirectionalSequence {
        let mut active = Value::Numeric(Default::default());
        code.0
            .iter()
            .map(|&value| {
                let diffs = get_direction_diffs_for_numeric_keypad(Value::Numeric(value), active);
                active = Value::Numeric(value);
                diffs
                    .iter()
                    .map(|directions| {
                        self.get_shortest_directions_to_press(
                            DirectionalSequence::from_directions(&directions),
                            robot_count - 1,
                        )
                    })
                    .min_by(|a, b| a.0.len().cmp(&b.0.len()))
                    .unwrap()
            })
            .fold(DirectionalSequence::default(), |mut acc, next| {
                acc.0.extend(next.0);
                acc
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
struct NumericValue(Option<u8>);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
struct DirectionalValue(Option<Direction>);

#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
struct NumericSequence(Vec<NumericValue>);
#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
struct DirectionalSequence(Vec<DirectionalValue>);

impl NumericSequence {
    pub fn numeric_code(&self) -> usize {
        format!("{}", self).replace("A", "").parse().unwrap()
    }
}

impl DirectionalSequence {
    pub fn from_directions(directions: &[Direction]) -> Self {
        Self(
            directions
                .iter()
                .copied()
                .map(Some)
                .chain([None])
                .map(DirectionalValue)
                .collect(),
        )
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Value {
    Directional(DirectionalValue),
    Numeric(NumericValue),
}

impl DirectionalSequence {
    pub fn iter_actions(&self) -> DirectionalSequenceActionsIterator {
        DirectionalSequenceActionsIterator::new(self)
    }
}

struct DirectionalSequenceActionsIterator<'a> {
    sequence: &'a DirectionalSequence,
    index: usize,
}

impl<'a> DirectionalSequenceActionsIterator<'a> {
    pub fn new(sequence: &'a DirectionalSequence) -> Self {
        Self { sequence, index: 0 }
    }
}

impl Iterator for DirectionalSequenceActionsIterator<'_> {
    type Item = DirectionalSequence;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.sequence.len() {
            let mut next = vec![self.sequence.0[self.index]];
            while self.sequence.0[self.index].0.is_some() && self.index < self.sequence.len() - 1 {
                self.index += 1;
                next.push(self.sequence.0[self.index]);
            }
            self.index += 1;
            Some(DirectionalSequence(next))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_directional_sequence_actions() {
        let sequence: DirectionalSequence = "v<<A>>^A<A>AvA<^AA>A<vAAA>^A".parse().unwrap();
        let len = sequence.iter_actions().count();
        println!("actions:");
        for action in sequence.iter_actions() {
            println!("{}", action);
        }
        assert_eq!(len, 12);
    }
}

impl NumericValue {
    pub fn as_point(&self) -> Point<usize> {
        match self.0 {
            None => Point { x: 2, y: 3 },
            Some(0) => Point { x: 1, y: 3 },
            Some(1) => Point { x: 0, y: 2 },
            Some(2) => Point { x: 1, y: 2 },
            Some(3) => Point { x: 2, y: 2 },
            Some(4) => Point { x: 0, y: 1 },
            Some(5) => Point { x: 1, y: 1 },
            Some(6) => Point { x: 2, y: 1 },
            Some(7) => Point { x: 0, y: 0 },
            Some(8) => Point { x: 1, y: 0 },
            Some(9) => Point { x: 2, y: 0 },
            _ => unreachable!(),
        }
    }
}

impl DirectionalValue {
    pub fn as_point(&self) -> Point<usize> {
        match self.0 {
            None => Point { x: 2, y: 0 },
            Some(Direction::Up) => Point { x: 1, y: 0 },
            Some(Direction::Down) => Point { x: 1, y: 1 },
            Some(Direction::Left) => Point { x: 0, y: 1 },
            Some(Direction::Right) => Point { x: 2, y: 1 },
        }
    }
}

impl Value {
    pub fn as_point(&self) -> Point<usize> {
        match self {
            Value::Directional(v) => v.as_point(),
            Value::Numeric(v) => v.as_point(),
        }
    }
}

#[derive(Clone, Default)]
struct DirectionDiffs {
    directions: Vec<Direction>,
    disallowed_starting_with: Option<&'static [Direction]>,
}

impl DirectionDiffs {
    const DISALLOWED_LEFT: &'static [Direction; 1] = &[Direction::Left];
    const DISALLOWED_LEFT_LEFT: &'static [Direction; 2] = &[Direction::Left, Direction::Left];
    const DISALLOWED_DOWN: &'static [Direction; 1] = &[Direction::Down];
    const DISALLOWED_DOWN_DOWN: &'static [Direction; 2] = &[Direction::Down, Direction::Down];
    const DISALLOWED_DOWN_DOWN_DOWN: &'static [Direction; 3] =
        &[Direction::Down, Direction::Down, Direction::Down];
    const DISALLOWED_UP: &'static [Direction; 1] = &[Direction::Up];

    pub fn disallow_left(&mut self) {
        self.disallowed_starting_with = Some(Self::DISALLOWED_LEFT);
    }
    pub fn disallow_left_left(&mut self) {
        self.disallowed_starting_with = Some(Self::DISALLOWED_LEFT_LEFT);
    }
    pub fn disallow_down(&mut self) {
        self.disallowed_starting_with = Some(Self::DISALLOWED_DOWN);
    }
    pub fn disallow_down_down(&mut self) {
        self.disallowed_starting_with = Some(Self::DISALLOWED_DOWN_DOWN);
    }
    pub fn disallow_down_down_down(&mut self) {
        self.disallowed_starting_with = Some(Self::DISALLOWED_DOWN_DOWN_DOWN);
    }
    pub fn disallow_up(&mut self) {
        self.disallowed_starting_with = Some(Self::DISALLOWED_UP);
    }

    pub fn iter(&self) -> impl Iterator<Item = Vec<Direction>> + '_ {
        self.directions
            .iter()
            .copied()
            .permutations(self.directions.len())
            .filter(|directions| {
                if let Some(disallowed) = self.disallowed_starting_with {
                    if directions.starts_with(disallowed) {
                        return false;
                    }
                }
                true
            })
    }
}

fn get_direction_diffs_for_numeric_keypad(target: Value, active: Value) -> DirectionDiffs {
    let (target, active) = (target.as_point(), active.as_point());
    let mut diffs = get_general_direction_diffs(&target, &active);

    if target.y < active.y && active.y == 3 && target.x == 0 {
        if active.x == 1 {
            diffs.disallow_left();
        } else {
            diffs.disallow_left_left();
        }
    }
    if target.x > active.x && active.x == 0 && target.y == 3 {
        match active.y {
            0 => {
                diffs.disallow_down_down_down();
            }
            1 => {
                diffs.disallow_down_down();
            }
            2 => {
                diffs.disallow_down();
            }
            _ => {}
        }
    }

    diffs
}

fn get_direction_diffs_for_directional_keypad(target: Value, active: Value) -> DirectionDiffs {
    let (target, active) = (target.as_point(), active.as_point());
    let mut diffs = get_general_direction_diffs(&target, &active);

    if target.y > active.y && target.x == 0 {
        if active.x == 1 {
            diffs.disallow_left();
        } else {
            diffs.disallow_left_left();
        }
    }
    if target.y < active.y && active.x == 0 {
        diffs.disallow_up();
    }

    diffs
}

fn get_general_direction_diffs(target: &Point<usize>, active: &Point<usize>) -> DirectionDiffs {
    let mut diffs = DirectionDiffs::default();

    if target.y < active.y {
        for _ in 0..active.y - target.y {
            diffs.directions.push(Direction::Up);
        }
    }
    if target.x > active.x {
        for _ in 0..target.x - active.x {
            diffs.directions.push(Direction::Right);
        }
    }
    if target.y > active.y {
        for _ in 0..target.y - active.y {
            diffs.directions.push(Direction::Down);
        }
    }
    if target.x < active.x {
        for _ in 0..active.x - target.x {
            diffs.directions.push(Direction::Left);
        }
    }

    diffs
}

impl Display for DirectionalSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|val| if let Some(val) = val.0 {
                    match val {
                        Direction::Up => '^',
                        Direction::Down => 'v',
                        Direction::Left => '<',
                        Direction::Right => '>',
                    }
                } else {
                    'A'
                })
                .collect::<String>()
        )
    }
}

impl FromStr for DirectionalSequence {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.chars()
                .map(|c| c.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl Display for DirectionalValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if let Some(val) = self.0 {
                match val {
                    Direction::Up => '^',
                    Direction::Down => 'v',
                    Direction::Left => '<',
                    Direction::Right => '>',
                }
            } else {
                'A'
            }
        )
    }
}

impl TryFrom<char> for DirectionalValue {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(Self(match value {
            'A' => None,
            '<' => Some(Direction::Left),
            '^' => Some(Direction::Up),
            '>' => Some(Direction::Right),
            'v' => Some(Direction::Down),
            other => return Err(anyhow::anyhow!("invalid character {}", other)),
        }))
    }
}

impl Display for NumericValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.0 {
            write!(f, "{}", value)
        } else {
            write!(f, "A")
        }
    }
}

impl TryFrom<char> for NumericValue {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(Self(match value {
            '0'..='9' => Some(value.to_digit(10).unwrap() as u8),
            'A' => None,
            other => {
                return Err(anyhow::anyhow!("invalid character {}", other));
            }
        }))
    }
}

impl FromStr for NumericSequence {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.chars()
                .map(|c| c.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl Display for NumericSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(|val| val.to_string()).collect::<String>()
        )
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["029A
980A
179A
456A
379A"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
