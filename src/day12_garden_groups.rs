use crate::common::models::{Direction, Grid, Point};
use crate::common::{Context, InputProvider};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let price = solve(input);
    println!("solution 1: {}", price);
    let price = solve2(input);
    println!("solution 2: {}", price);
    let price = solve2v2(input);
    println!("solution 2: {}", price);
}

fn solve(input: &str) -> usize {
    let mut total_price = 0usize;
    let plot: Plot = input.parse().unwrap();
    let mut used: HashSet<Point<usize>> = HashSet::new();
    for (point, &value) in plot.0.iter() {
        if used.contains(&point) {
            continue;
        }
        let mut members = 0;
        let mut edges = 0;
        let mut current = Vec::from([point]);
        while !current.is_empty() {
            let mut next = Vec::new();
            for current in current {
                if used.contains(&current) {
                    continue;
                } else {
                    used.insert(current.clone());
                }
                members += 1;

                let neighbors = plot.0.four_way_neighbors(&current);
                edges += 4 - neighbors.len();
                for p in neighbors {
                    let v = plot.0.get(&p).copied().unwrap();
                    if v == value {
                        next.push(p);
                    } else {
                        edges += 1;
                    }
                }
            }

            current = next;
        }

        let price = members * edges;
        log::debug!(
            "A region of {} plants with price {} * {} = {}.",
            value,
            members,
            edges,
            price
        );
        total_price += price;
    }

    total_price
}
fn solve2(input: &str) -> usize {
    let plot: Plot = input.parse().unwrap();
    let mut used: HashSet<Point<usize>> = HashSet::new();

    let mut regions: Vec<Region> = Vec::new();

    for (point, _) in plot.0.iter() {
        if used.contains(&point) {
            continue;
        }
        let region = Region::build(&plot, point);

        used.extend(region.members.clone());
        regions.push(region);
    }

    let mut total_price = 0usize;
    for mut region in regions.clone() {
        for other_region in &regions {
            if !other_region.contained_in.is_empty()
                && other_region
                    .contained_in
                    .iter()
                    .all(|p| region.members.contains(p))
            {
                log::debug!(
                    "Region {} is inside region {}",
                    other_region.name,
                    region.name
                );
                region.sides_inside += other_region.sides;
            }
        }

        let price = region.members.len() * (region.sides + region.sides_inside);
        log::debug!(
            "A region of {} plants with price {} * ({} + {}) = {}.",
            region.name,
            region.members.len(),
            region.sides,
            region.sides_inside,
            price
        );

        total_price += price;
    }

    total_price
}

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
struct Edge {
    position: Point<usize>,
    direction: Direction,
}

impl Debug for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.position,
            match self.direction {
                Direction::Up => "^",
                Direction::Down => "v",
                Direction::Left => "<",
                Direction::Right => ">",
            }
        )
    }
}

fn solve2v2(input: &str) -> usize {
    let mut total_price = 0usize;
    let plot: Plot = input.parse().unwrap();
    let mut used: HashSet<Point<usize>> = HashSet::new();
    const DIRECTIONS: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];

    fn consolidate_edges(edges: &mut HashSet<Edge>, edge: &Edge, plot: &Plot, value: char) {
        for &direction_func in &[Direction::turn_left, Direction::turn_right] {
            let mut current = edge.position.clone();
            let direction = direction_func(&edge.direction);
            'inner: loop {
                if let Some(p) = plot.0.try_move_if(&current, direction, |&v| v == value) {
                    if !edges.remove(&Edge {
                        position: p.clone(),
                        direction: edge.direction,
                    }) {
                        break 'inner;
                    }
                    current = p;
                } else {
                    break 'inner;
                }
            }
        }
    }

    for (point, &value) in plot.0.iter() {
        if used.contains(&point) {
            continue;
        }

        let mut members = 0;
        let mut edges: HashSet<Edge> = Default::default();
        let mut current_points = vec![point];

        while !current_points.is_empty() {
            let mut next_points = Vec::new();
            for current_point in current_points {
                if used.contains(&current_point) {
                    continue;
                }
                used.insert(current_point.clone());
                members += 1;
                for &direction in &DIRECTIONS {
                    if let Some(p) = plot
                        .0
                        .try_move_if(&current_point, direction, |&v| v == value)
                    {
                        next_points.push(p);
                    } else {
                        edges.insert(Edge {
                            position: current_point.clone(),
                            direction,
                        });
                    }
                }
            }
            current_points = next_points;
        }

        for edge in edges.clone() {
            if edges.contains(&edge) {
                consolidate_edges(&mut edges, &edge, &plot, value);
            }
        }

        let price = members * edges.len();
        log::trace!("{}: edges: {:?}", value, edges);
        log::debug!(
            "A region of {} plants with price {} * {} = {}.",
            value,
            members,
            edges.len(),
            price
        );
        total_price += price;
    }

    total_price
}

#[derive(Clone)]
struct Region {
    name: char,
    members: HashSet<Point<usize>>,
    sides: usize,
    sides_inside: usize,
    contained_in: HashSet<Point<usize>>,
}

impl Region {
    pub fn build(plot: &Plot, mut start: Point<usize>) -> Self {
        let value = *plot.0.get(&start).unwrap();

        'init: loop {
            let directions = [Direction::Up, Direction::Left];
            for direction in directions {
                if let Some(next) = start.move_to(direction) {
                    if plot.0.get(&next) == Some(&value) {
                        start = next;
                        continue 'init;
                    }
                }
            }
            break;
        }

        let mut sides = 0;
        let members = Self::get_members(plot, start.clone());
        log::trace!("{}: members: {:?}", value, members);
        log::trace!("{}: start: {}", value, start);
        let start = start;
        let start_direction = Direction::Right;
        let mut current = start.clone();
        let mut direction = Direction::Right;

        let mut enclosed = true;
        let mut contained_in = HashSet::new();

        loop {
            if sides >= 4 && current == start && direction == start_direction {
                break;
            }
            if let Some(next) = current.move_to(direction) {
                log::trace!("{}: trying to move to {}", value, next);
                if members.contains(&next) {
                    log::trace!("{}: {} is a member", value, next);
                    current = next;
                    let perpendicular = direction.turn_left();
                    if let Some(left) = current.move_to(perpendicular) {
                        if plot.0.get(&left) == Some(&value) {
                            direction = perpendicular;
                            sides += 1;
                            log::trace!(
                                "{}: ++side = {} moving in perpendicular direction {:?} to {}",
                                value,
                                sides,
                                perpendicular,
                                current
                            );
                        } else {
                            contained_in.insert(left);
                        }
                    } else {
                        enclosed = false;
                    }

                    continue;
                } else {
                    contained_in.insert(next);
                }
            } else {
                enclosed = false;
            }
            direction = direction.turn_right();
            sides += 1;
            log::trace!("{}: ++side = {} turning to {:?}", value, sides, direction);
        }

        if !enclosed {
            contained_in = Default::default();
        }

        Self {
            name: value,
            members,
            sides,
            sides_inside: 0,
            contained_in,
        }
    }
    fn get_members(plot: &Plot, start: Point<usize>) -> HashSet<Point<usize>> {
        let value = *plot.0.get(&start).unwrap();
        let mut members = HashSet::new();

        let mut current = Vec::from([start]);
        while !current.is_empty() {
            let mut next = Vec::new();
            for current in current {
                if members.contains(&current) {
                    continue;
                } else {
                    members.insert(current.clone());
                }

                let neighbors = plot.0.four_way_neighbors(&current);
                for p in neighbors {
                    let v = plot.0.get(&p).copied().unwrap();
                    if v == value {
                        next.push(p);
                    }
                }
            }

            current = next;
        }

        members
    }
}

#[derive(Clone)]
struct Plot(Grid<char>);

impl FromStr for Plot {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Grid::from_iter(s.lines().map(|line| line.chars()))))
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE",
        "AAAA
BBCD
BBCC
EEEC",
        "EEEEE
EXXXX
EEEEE
EXXXX
EEEEE",
        "AAAAAA
AAABBA
AAABBA
ABBAAA
ABBAAA
AAAAAA",
        "OOOOO
OXOXO
OOOOO
OXOXO
OOOOO",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
