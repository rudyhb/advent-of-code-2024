use crate::common::models::Point;
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let robots: Vec<Robot> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut space = if context.is_testing() {
        Space::new_testing()
    } else {
        Space::new()
    };

    space.insert_robots(robots.into_iter());
    let original_space = space.clone();
    let mut safety_factors = Vec::new();

    log::debug!("initial: {}", space);
    for i in 0..100 {
        space.tick();
        if i <= 5 {
            log::debug!("after {} seconds: {}", i, space);
        }
        safety_factors.push(space.get_safety_factor());
    }
    log::debug!("after 100 seconds: {}", space);
    println!("safety factor: {}", space.get_safety_factor());

    let avg_safety = safety_factors.iter().copied().sum::<usize>() / safety_factors.len();
    println!("avg safety factor: {}", avg_safety);

    let mut min_safety = avg_safety;
    let mut min_safety_seconds = 0;
    for i in 101..10_000 {
        space.tick();

        let safety = space.get_safety_factor();
        if safety < min_safety {
            min_safety = safety;
            min_safety_seconds = i;
        }
        safety_factors.push(safety);
    }
    println!(
        "min safety factor {} at {} seconds",
        min_safety, min_safety_seconds
    );
    safety_factors.sort();
    println!("min safety factors: {:?}", &safety_factors[0..10]);
    
    let mut space = original_space;
    for i in 1..10_000 {
        space.tick();
        let safety = space.get_safety_factor();
        if safety_factors[0..10].contains(&safety) {
            println!("after {} seconds: {}", i, space);
        }
    }
}

#[derive(Clone)]
struct Robot {
    start_position: Point<i64>,
    velocity: Point<i64>,
}

#[derive(Clone)]
struct Space {
    robots: HashMap<Point<i64>, Vec<Robot>>,
    size_x: i64,
    size_y: i64,
}

impl Space {
    pub fn new() -> Self {
        Self {
            robots: Default::default(),
            size_x: 101,
            size_y: 103,
        }
    }
    pub fn new_testing() -> Self {
        Self {
            robots: Default::default(),
            size_x: 11,
            size_y: 7,
        }
    }
    pub fn insert_robots(&mut self, robots: impl Iterator<Item = Robot>) {
        for robot in robots {
            self.insert_robot(robot.start_position.clone(), robot);
        }
    }
    fn insert_robot(&mut self, at: Point<i64>, robot: Robot) {
        self.robots.entry(at).or_default().push(robot);
    }
    pub fn tick(&mut self) {
        for (point, robots) in std::mem::take(&mut self.robots) {
            for robot in robots {
                let mut point = &point + &robot.velocity;
                point.x = (point.x + self.size_x) % self.size_x;
                point.y = (point.y + self.size_y) % self.size_y;
                self.insert_robot(point, robot);
            }
        }
    }
    pub fn get_safety_factor(&self) -> usize {
        let mut quadrants = Quadrant::get_quadrants(self);

        for (point, robots) in &self.robots {
            for quadrant in quadrants.iter_mut() {
                if quadrant.try_insert(point, robots.len()) {
                    break;
                }
            }
        }

        for quadrant in quadrants.iter() {
            log::debug!("quadrant {:?}: {}", quadrant.quadrant_type, quadrant.count);
        }

        quadrants.into_iter().map(|q| q.count).product()
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size_y {
            write!(f, "\n")?;
            for x in 0..self.size_x {
                let point = Point { x, y };
                if let Some(robots) = self.robots.get(&point) {
                    write!(f, "{}", robots.len())?;
                } else {
                    write!(f, ".")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum QuadrantType {
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

struct Quadrant<'a> {
    quadrant_type: QuadrantType,
    space: &'a Space,
    count: usize,
}

impl<'a> Quadrant<'a> {
    fn new(quadrant_type: QuadrantType, space: &'a Space) -> Self {
        Self {
            quadrant_type,
            space,
            count: 0,
        }
    }
    pub fn get_quadrants(for_space: &'a Space) -> [Self; 4] {
        [
            Quadrant::new(QuadrantType::NorthWest, for_space),
            Quadrant::new(QuadrantType::NorthEast, for_space),
            Quadrant::new(QuadrantType::SouthWest, for_space),
            Quadrant::new(QuadrantType::SouthEast, for_space),
        ]
    }
    pub fn try_insert(&mut self, item: &Point<i64>, count: usize) -> bool {
        if match &self.quadrant_type {
            QuadrantType::NorthWest => {
                item.x < self.space.size_x / 2 && item.y < self.space.size_y / 2
            }
            QuadrantType::NorthEast => {
                item.x > self.space.size_x / 2 && item.y < self.space.size_y / 2
            }
            QuadrantType::SouthWest => {
                item.x < self.space.size_x / 2 && item.y > self.space.size_y / 2
            }
            QuadrantType::SouthEast => {
                item.x > self.space.size_x / 2 && item.y > self.space.size_y / 2
            }
        } {
            self.count += count;
            true
        } else {
            false
        }
    }
}

impl FromStr for Robot {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Result<Regex, regex::Error>> =
            Lazy::new(|| Regex::new(r"p=(?<px>-?\d+),(?<py>-?\d+) v=(?<vx>-?\d+),(?<vy>-?\d+)"));

        match &*RE {
            Ok(re) => {
                let captures = re.captures(s).context("invalid robot input line")?;
                Ok(Robot {
                    start_position: Point {
                        x: captures["px"].parse()?,
                        y: captures["py"].parse()?,
                    },
                    velocity: Point {
                        x: captures["vx"].parse()?,
                        y: captures["vy"].parse()?,
                    },
                })
            }
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
