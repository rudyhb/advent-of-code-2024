use crate::common::models::{Direction, Point};
use crate::common::{Context, InputProvider};
use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utils::a_star::{a_star_search, Node, Successor};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let door_codes: Vec<NumericSequence> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let sum_complexities: usize = door_codes
        .into_iter()
        .map(|code| {
            let best = solve(&code, 3);
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

    println!("sum complexities: {}", sum_complexities);
}

#[derive(Clone, Debug, Hash)]
struct State {
    last_value: Value,
    input_iterator: SequenceIterator,
    current_output: DirectionalSequence,
    robot_index: usize,
}

impl State {
    pub fn try_next_robot(&mut self, robot_count: usize) -> bool {
        self.input_iterator =
            Sequence::Directional(std::mem::take(&mut self.current_output)).into_iter();
        self.last_value = Value::Directional(Default::default());
        self.robot_index += 1;

        self.robot_index < robot_count
    }
}

impl Node for State {}

fn solve(door_code: &NumericSequence, robot_count: usize) -> DirectionalSequence {
    assert!(robot_count > 1);
    let start = State {
        last_value: Value::Numeric(Default::default()),
        input_iterator: Sequence::Numeric(door_code.clone()).into_iter(),
        current_output: Default::default(),
        robot_index: 0,
    };

    let distance_estimate = |state: &State| -> usize { robot_count - state.robot_index - 1 };

    let get_successors = |state: &State| -> Vec<Successor<State, usize>> {
        let mut state = state.clone();
        let button = loop {
            if let Some(button) = state.input_iterator.next() {
                break button;
            } else if !state.try_next_robot(robot_count) {
                return vec![];
            }
        };
        let direction_possibilities = if state.robot_index == 0 {
            get_direction_diffs_for_numeric_keypad(button, state.last_value)
        } else {
            get_direction_diffs_for_directional_keypad(button, state.last_value)
        };

        direction_possibilities
            .iter()
            .map(|possibility| {
                let mut state = state.clone();
                let sequence = DirectionalSequence::from_directions(&possibility);
                let cost = sequence.0.len();
                state.current_output.extend(sequence);
                state.last_value = button;
                Successor::new(state, cost)
            })
            .collect()
    };

    let result = a_star_search(
        start,
        get_successors,
        |details| distance_estimate(&details.current_node),
        |state| state.robot_index == robot_count - 1 && state.input_iterator.is_at_end(),
        None,
    )
    .unwrap();

    result.shortest_path.last().cloned().unwrap().current_output
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
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, index: usize) -> Value {
        Value::Numeric(self.0[index])
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
    pub fn get(&self, index: usize) -> Value {
        Value::Directional(self.0[index])
    }
    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Value {
    Directional(DirectionalValue),
    Numeric(NumericValue),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Sequence {
    Directional(DirectionalSequence),
    #[allow(unused)]
    Numeric(NumericSequence),
}

impl Sequence {
    pub fn len(&self) -> usize {
        match self {
            Sequence::Directional(s) => s.len(),
            Sequence::Numeric(s) => s.len(),
        }
    }
    pub fn get(&self, index: usize) -> Value {
        match self {
            Sequence::Directional(s) => s.get(index),
            Sequence::Numeric(s) => s.get(index),
        }
    }
    pub fn into_iter(self) -> SequenceIterator {
        SequenceIterator {
            sequence: self,
            index: 0,
        }
    }
}

#[derive(Clone, Debug, Hash)]
struct SequenceIterator {
    sequence: Sequence,
    index: usize,
}

impl SequenceIterator {
    pub fn is_at_end(&self) -> bool {
        self.index >= self.sequence.len()
    }
}

impl Iterator for SequenceIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.sequence.len() {
            let val = self.sequence.get(self.index);
            self.index += 1;
            Some(val)
        } else {
            None
        }
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(
            s.chars()
                .map(|c| c.try_into())
                .collect::<std::result::Result<Vec<_>, _>>()?,
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(
            s.chars()
                .map(|c| c.try_into())
                .collect::<std::result::Result<Vec<_>, _>>()?,
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

impl Display for Sequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Sequence::Directional(s) => {
                write!(f, "{}", s)
            }
            Sequence::Numeric(s) => {
                write!(f, "{}", s)
            }
        }
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
