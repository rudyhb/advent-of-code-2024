use crate::common::models::{MultiplyByI64, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let anti_nodes = count_anti_nodes(input, false);
    println!("Part 1: {}", anti_nodes);
    let anti_nodes = count_anti_nodes(input, true);
    println!("Part 2: {}", anti_nodes);
}

fn count_anti_nodes(input: &str, is_v2: bool) -> usize {
    let mut grid: Grid = input.parse().unwrap();
    if is_v2 {
        grid.calculate_anti_nodes_v2();
    } else {
        grid.calculate_anti_nodes();
    }
    log::debug!("grid with anti nodes:{}", grid);
    log::debug!("anti nodes:");
    for (&c, n) in &grid.anti_nodes {
        log::debug!("anti node {}: {:?}", c, n);
    }
    grid.anti_nodes
        .iter()
        .flat_map(|(_, v)| v)
        .collect::<HashSet<_>>()
        .len()
}

struct Grid {
    antennas: HashMap<char, Vec<Point<i64>>>,
    anti_nodes: HashMap<char, Vec<Point<i64>>>,
    size_x: usize,
    size_y: usize,
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size_y {
            write!(f, "\n")?;
            'x: for x in 0..self.size_x {
                let point = Point {
                    x: x as i64,
                    y: y as i64,
                };
                for (&c, antennas) in &self.antennas {
                    if antennas.iter().any(|a| a == &point) {
                        write!(f, "{}", c)?;
                        continue 'x;
                    }
                }
                if self
                    .anti_nodes
                    .iter()
                    .any(|n| n.1.iter().any(|p| p == &point))
                {
                    write!(f, "#")?;
                }

                write!(f, ".")?;
            }
        }
        Ok(())
    }
}

impl Grid {
    fn in_grid(&self, point: &Point<i64>) -> bool {
        point.x >= 0 && point.y >= 0 && point.x < self.size_x as i64 && point.y < self.size_y as i64
    }
    pub fn calculate_anti_nodes(&mut self) {
        let mut anti_nodes = HashMap::new();
        for (&char, antennas) in self.antennas.iter() {
            for i in 0..antennas.len() {
                for j in i + 1..antennas.len() {
                    log::debug!(
                        "antenna i: {:?}, antenna j: {:?}, diff: {:?}",
                        antennas[i],
                        antennas[j],
                        &antennas[i] - &antennas[j]
                    );
                    let diff = &antennas[i] - &antennas[j];

                    let point = &antennas[i] + &diff;
                    if self.in_grid(&point) {
                        anti_nodes.entry(char).or_insert_with(Vec::new).push(point);
                    }

                    let point = &antennas[j] - &diff;
                    if self.in_grid(&point) {
                        anti_nodes.entry(char).or_insert_with(Vec::new).push(point);
                    }
                }
            }
        }

        self.anti_nodes = anti_nodes
    }
    pub fn calculate_anti_nodes_v2(&mut self) {
        let mut anti_nodes = HashMap::new();
        for (&char, antennas) in self.antennas.iter() {
            for i in 0..antennas.len() {
                for j in i + 1..antennas.len() {
                    log::debug!(
                        "antenna i: {:?}, antenna j: {:?}, diff: {:?}",
                        antennas[i],
                        antennas[j],
                        &antennas[i] - &antennas[j]
                    );
                    let diff = &antennas[i] - &antennas[j];
                    
                    let entry = anti_nodes.entry(char).or_insert_with(Vec::new);
                    
                    entry.push(antennas[j].clone());
                    let mut k = 1;
                    loop {
                        let point = &antennas[j] - &diff.multiply_by_i64(k);
                        if self.in_grid(&point) {
                            entry.push(point);
                        } else {
                            break;
                        }
                        k += 1;
                    }

                    k = 1;
                    entry.push(antennas[i].clone());
                    loop {
                        let point = &antennas[i] + &diff.multiply_by_i64(k);
                        if self.in_grid(&point) {
                            entry.push(point);
                        } else {
                            break;
                        }
                        k += 1;
                    }
                }
            }
        }

        self.anti_nodes = anti_nodes
    }
}

impl FromStr for Grid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut y = 0usize;
        let mut size_x = None;
        let mut antennas = HashMap::new();

        for line in s.lines() {
            let mut x = 0usize;
            for c in line.chars() {
                match c {
                    'A'..='z' | '0'..='9' => {
                        let point = Point {
                            x: x as i64,
                            y: y as i64,
                        };
                        antennas.entry(c).or_insert_with(Vec::new).push(point);
                    }
                    '.' => {}
                    other => {
                        return Err(anyhow::anyhow!("invalid character: {}", other));
                    }
                }
                x += 1;
            }

            if let Some(size_x) = size_x {
                if x != size_x {
                    return Err(anyhow::anyhow!(
                        "grid is not even on x: {} vs {} for row {}",
                        size_x,
                        x,
                        y
                    ));
                }
            } else {
                size_x = Some(x);
            }
            y += 1;
        }

        Ok(Self {
            antennas,
            anti_nodes: Default::default(),
            size_y: y,
            size_x: size_x.context("empty grid")?,
        })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
