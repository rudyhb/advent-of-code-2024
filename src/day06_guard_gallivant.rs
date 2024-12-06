use crate::common::models::{Direction, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let visited = solve(input);
    println!("the guard visited {} places", visited);

    let obstacles = solve_v2(input);
    println!("obstacles can be placed in {} places", obstacles);
}

fn solve(input: &str) -> usize {
    let mut map: Map = input.parse().unwrap();
    log::debug!("{}", map);
    while map.next().unwrap() {}
    log::debug!("guard exited at location: {:?}", map.guard.position);
    log::debug!("{}", map);
    map.visited.len()
}

fn solve_v2(input: &str) -> usize {
    let ref_map: Map = input.parse().unwrap();
    let mut map = ref_map.clone();
    while map.next().unwrap() { }
    let mut new_obstacles = map.visited;
    new_obstacles.remove(&ref_map.guard.position);
    
    let mut result = 0usize;
    for obstacle in new_obstacles {
        let mut map = ref_map.clone();
        map.obstacles.insert(obstacle);
        if map.does_loop().unwrap() {
            result += 1;
        }
    }
    
    result
}

#[derive(Clone)]
struct Map {
    size_x: usize,
    size_y: usize,
    guard: GuardPosition,
    obstacles: HashSet<Point>,
    visited: HashSet<Point>,
}

#[derive(Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Debug)]
struct GuardPosition {
    position: Point,
    direction: Direction,
}

impl GuardPosition {
    pub fn new(position: Point, direction: Direction) -> Self {
        Self {
            position,
            direction,
        }
    }
}


impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size_y {
            write!(f, "\n")?;
            for x in 0..self.size_x {
                let point = Point { x, y };
                if self.obstacles.contains(&point) {
                    write!(f, "#")?;
                } else if self.guard.position == point {
                    write!(
                        f,
                        "{}",
                        match self.guard.direction {
                            Direction::Up => "^",
                            Direction::Down => "v",
                            Direction::Left => "<",
                            Direction::Right => ">",
                        }
                    )?;
                } else if self.visited.contains(&point) {
                    write!(f, "X")?;
                } else {
                    write!(f, ".")?;
                }
            }
        }
        Ok(())
    }
}

impl Map {
    pub fn change_direction_and_get_next(&mut self) -> anyhow::Result<Option<Point>> {
        let mut i = 0;
        let next = loop {
            if i >= 4 {
                return Err(anyhow::anyhow!("guard is stuck turning!!"));
            }
            if let Some(next) = self.guard.position.move_to(self.guard.direction) {
                if next.x >= self.size_x || next.y >= self.size_y {
                    return Ok(None); // left map
                }
                if !self.obstacles.contains(&next) {
                    break next;
                }
            } else {
                return Ok(None); // left map
            }

            if i == 0 {
                log::trace!("guard hit obstacle at location: {:?}", self.guard.position);
                log::trace!("{}", self);
            }

            self.guard.direction = self.guard.direction.turn_right();
            i += 1;
        };

        Ok(Some(next))
    }
    fn walk_to(&mut self, next: Point) {
        self.visited.insert(next.clone());
        self.guard.position = next;
    }
    pub fn next(&mut self) -> anyhow::Result<bool> {
        if let Some(next) = self.change_direction_and_get_next()? {
            self.walk_to(next);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn does_loop(&mut self) -> anyhow::Result<bool> {
        let mut already_there: HashSet<GuardPosition> = Default::default();
        while self.next()? {
            if already_there.contains(&self.guard) {
                return Ok(true);
            }
            already_there.insert(self.guard.clone());
        }

        Ok(false)
    }
}

impl FromStr for Map {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut size_x: Option<usize> = None;
        let mut size_y = 0usize;
        let mut obstacles = HashSet::new();
        let mut guard_position = None;
        let mut guard_heading = None;

        for line in s.lines() {
            let mut x = 0usize;
            for c in line.chars() {
                let point = Point { x, y: size_y };
                match c {
                    '#' => {
                        obstacles.insert(point);
                    }
                    'v' => {
                        guard_position = Some(point);
                        guard_heading = Some(Direction::Down);
                    }
                    '>' => {
                        guard_position = Some(point);
                        guard_heading = Some(Direction::Right);
                    }
                    '^' => {
                        guard_position = Some(point);
                        guard_heading = Some(Direction::Up);
                    }
                    '<' => {
                        guard_position = Some(point);
                        guard_heading = Some(Direction::Left);
                    }
                    '.' => {}
                    other => {
                        return Err(anyhow::anyhow!(
                            "Unexpected character {} in line {}",
                            other,
                            size_y
                        ))
                    }
                }
                x += 1;
            }

            if let Some(size_x) = size_x {
                if x != size_x {
                    return Err(anyhow::anyhow!(
                        "Expected constant line length of {} but got {} in line {}",
                        size_x,
                        x,
                        size_y
                    ));
                }
            } else {
                size_x = Some(x);
            }
            size_y += 1;
        }

        let guard_position = guard_position.context("guard not found")?;
        let visited = HashSet::from([guard_position.clone()]);

        Ok(Map {
            size_x: size_x.context("invalid size x")?,
            size_y,
            guard: GuardPosition::new(guard_position, guard_heading.context("guard not found")?),
            obstacles,
            visited,
        })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#..."]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
