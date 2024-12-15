use crate::common::models::{Grid, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let mut map: Map = input.parse().unwrap();
    map.fill_path_scores();
    println!("solution 1: {}", map.trailhead_scores_sum());
    println!("solution 2: {}", map.trailhead_ratings_sum());
}

struct Map {
    grid: Grid<u8>,
    path_scores: HashMap<Point<usize>, HashSet<Point<usize>>>,
    path_ratings: HashMap<Point<usize>, u32>,
}

impl Map {
    pub fn new(grid: Grid<u8>) -> Self {
        Self {
            grid,
            path_scores: Default::default(),
            path_ratings: Default::default(),
        }
    }
    fn get_trailheads(&self) -> impl Iterator<Item = Point<usize>> + use<'_> {
        self.grid
            .iter()
            .filter(|(_, &value)| value == 0)
            .map(|(p, _)| p)
    }
    pub fn fill_path_scores(&mut self) {
        for point in self.get_trailheads().collect::<Vec<_>>() {
            self.fill_next_path(point.clone(), 1);
            self.fill_next_path_ratings(point, 1);
        }
    }
    fn fill_next_path(&mut self, point: Point<usize>, next_value: u8) -> HashSet<Point<usize>> {
        if let Some(cached) = self.path_scores.get(&point) {
            return cached.clone();
        }

        let mut score = HashSet::new();
        for next in self
            .grid
            .four_way_neighbors(&point)
            .into_iter()
            .filter(|p| self.grid.get(p) == Some(&next_value))
            .collect::<Vec<_>>()
        {
            if next_value == 9 {
                score.insert(next);
            } else {
                score.extend(self.fill_next_path(next, next_value + 1));
            }
        }

        log::trace!("resolved {} {} = {}", next_value - 1, point, score.len());
        self.path_scores.entry(point).or_insert(score).clone()
    }
    fn fill_next_path_ratings(&mut self, point: Point<usize>, next_value: u8) -> u32 {
        if let Some(cached) = self.path_ratings.get(&point) {
            return *cached;
        }

        let score = self
            .grid
            .four_way_neighbors(&point)
            .into_iter()
            .filter(|p| self.grid.get(p) == Some(&next_value))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|next| {
                if next_value == 9 {
                    1
                } else {
                    self.fill_next_path_ratings(next, next_value + 1)
                }
            })
            .sum();

        log::trace!("ratings resolved {} {} = {}", next_value - 1, point, score);
        self.path_ratings.insert(point, score);
        score
    }
    pub fn trailhead_scores_sum(&self) -> usize {
        self.get_trailheads()
            .map(|t| {
                self.path_scores
                    .get(&t)
                    .expect("trailhead path not calculated")
                    .len()
            })
            .sum()
    }
    pub fn trailhead_ratings_sum(&self) -> u32 {
        self.get_trailheads()
            .map(|t| {
                self.path_ratings
                    .get(&t)
                    .copied()
                    .expect("trailhead path not calculated")
            })
            .sum()
    }
}

impl FromStr for Map {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Grid::try_from_iter(s.lines().map(|line| {
            line.chars()
                .map(|c| c.to_digit(10).context("invalid character").map(|d| d as u8))
        }))?))
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
