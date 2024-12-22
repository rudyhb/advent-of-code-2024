use crate::common::models::{Direction, Grid, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use utils::a_star::{self, a_star_search, ComputationResult, Options, Successor};

//noinspection RsConstantConditionIf
pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let racetrack: Racetrack = input.parse().unwrap();
    let picoseconds = racetrack.solve_simple(&Default::default()).unwrap();
    println!("fastest time no cheating: {}", picoseconds);

    if false {
        let savings = if context.is_testing() {
            picoseconds - 1
        } else {
            picoseconds - 100
        };

        let mut total_that_save = 0;
        let solutions_with_cheating = racetrack.solve_all_cheating(savings).unwrap();
        for (&pico, &count) in solutions_with_cheating.iter().rev() {
            total_that_save += count;
            println!(
                "There are {} cheats that save {} picoseconds.",
                count,
                picoseconds - pico
            );
        }
        println!("there are {} that save picoseconds", total_that_save);
    }

    println!("--------\nPART 2---------\n");

    let savings = if context.is_testing() {
        picoseconds - 50
    } else {
        picoseconds - 100
    };

    let mut total_that_save = 0;
    let solutions_with_cheating = racetrack.solve_all_cheating_v2(savings).unwrap();
    for (&pico, &count) in solutions_with_cheating.iter().rev() {
        total_that_save += count;
        println!(
            "There are {} cheats that save {} picoseconds.",
            count,
            picoseconds - pico
        );
    }
    println!("there are {} that save picoseconds", total_that_save);
}

enum Space {
    Empty,
    Wall,
}

impl Default for Space {
    fn default() -> Self {
        Self::Empty
    }
}

struct Racetrack {
    grid: Grid<Space>,
    start: Point<usize>,
    end: Point<usize>,
}

impl FromStr for Racetrack {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut start = None;
        let mut end = None;
        let grid = Grid::from_str_with(s, |c, point| {
            Ok(match c {
                '#' => Some(Space::Wall),
                '.' => None,
                'S' => {
                    start = Some(point.clone());
                    None
                }
                'E' => {
                    end = Some(point.clone());
                    None
                }
                other => return Err(anyhow::anyhow!("invalid character {}", other)),
            })
        })?;
        Ok(Self {
            grid,
            start: start.context("no start position")?,
            end: end.context("no end position")?,
        })
    }
}

impl Racetrack {
    pub fn solve(
        &self,
        ignore_spaces: &HashSet<Point<usize>>,
    ) -> a_star::Result<ComputationResult<Point<usize>, usize>> {
        let start = self.start.clone();
        let end = &self.end;
        a_star_search(
            start,
            end,
            |node| {
                Direction::directions()
                    .into_iter()
                    .filter_map(|direction| {
                        self.grid.try_move_if(node, direction, |pos, value| {
                            !matches!(value, Space::Wall) || ignore_spaces.contains(pos)
                        })
                    })
                    .map(|new_point| Successor::new(new_point, 1))
                    .collect()
            },
            |details| details.current_node.manhattan_distance(details.target_node),
            |left, right| left == right,
            None,
        )
    }
    pub fn solve_simple(&self, ignore_spaces: &HashSet<Point<usize>>) -> a_star::Result<usize> {
        self.solve(ignore_spaces)
            .map(|result| result.shortest_path_cost)
    }
    pub fn solve_all_cheating(&self, max_score: usize) -> anyhow::Result<BTreeMap<usize, usize>> {
        let solutions = self
            .grid
            .iter()
            .filter(|(_, value)| matches!(value, Space::Wall))
            .filter_map(|(ignore_space, _)| {
                let set = HashSet::from([ignore_space.clone()]);
                self.solve_simple(&set).ok().and_then(|score| {
                    if score <= max_score {
                        Some((score, ignore_space))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        Ok(solutions
            .into_iter()
            .map(|(cost, ignore_space)| {
                log::trace!(
                    "solution with cheating that costs {}:\n{}",
                    cost,
                    self.grid
                        .display_with_overrides([(&ignore_space, &'1')].into_iter().collect())
                );
                cost
            })
            .fold(BTreeMap::default(), |mut acc, next| {
                *acc.entry(next).or_default() += 1;
                acc
            }))
    }
    pub fn solve_all_cheating_v2(
        &self,
        max_score: usize,
    ) -> anyhow::Result<BTreeMap<usize, usize>> {
        let solution_path = self.solve(&HashSet::default())?.shortest_path;
        let wormholes: Vec<_> = 
        (0..solution_path.len() - 1)
            .flat_map(|i| (i + 1..solution_path.len())
                .map(move |j| (i, j))
            )
            .filter_map(|(i, j)| Wormhole::try_from(&solution_path[i], &solution_path[j]))
            .collect();
        log::debug!("working with {} wormholes", wormholes.len());
        
        let results: Vec<_> = wormholes.into_par_iter()
            .filter_map(|wormhole| {
                self.solve_with_wormhole(max_score, wormhole).ok()
            })
            .collect();
        let result = results.into_iter()
            .fold(BTreeMap::new(), |mut acc, next| {
                *acc.entry(next).or_default() += 1;
                acc
            });
            
        Ok(result)
    }
    fn solve_with_wormhole(&self, max_score: usize, wormhole: Wormhole) -> anyhow::Result<usize> {
        let start = self.start.clone();
        let end = &self.end;

        let mut total = self.solve_stage(start, &wormhole.from)?;
        if total > max_score {
            return Err(anyhow::anyhow!("too high"));
        }
        total += wormhole.get_trip_time();
        if total > max_score {
            return Err(anyhow::anyhow!("too high"));
        }
        total += self.solve_stage(wormhole.to.clone(), end)?;
        if total > max_score {
            return Err(anyhow::anyhow!("too high"));
        }

        Ok(total)
    }
    fn solve_stage(&self, from: Point<usize>, to: &Point<usize>) -> anyhow::Result<usize> {
        if &from == to {
            return Ok(0);
        }
        let result = a_star_search(
            from,
            to,
            |node| {
                Direction::directions()
                    .into_iter()
                    .filter_map(|direction| {
                        self.grid
                            .try_move_if(node, direction, |_, value| !matches!(value, Space::Wall))
                    })
                    .map(|new_point| Successor::new(new_point, 1))
                    .collect()
            },
            |details| {
                details
                    .current_node
                    .manhattan_distance(details.target_node)
            },
            |left, right| left == right,
            Some(&Options::default().with_no_logs()),
        )?;
        Ok(result.shortest_path_cost)
    }
}

struct Wormhole {
    from: Point<usize>,
    to: Point<usize>,
}

impl Wormhole {
    pub fn try_from(from: &Point<usize>, to: &Point<usize>) -> Option<Self> {
        if from.manhattan_distance(to) <= 20 {
            Some(Self {
                from: from.clone(),
                to: to.clone(),
            })
        } else {
            None
        }
    }
    pub fn get_trip_time(&self) -> usize {
        self.from.manhattan_distance(&self.to)
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Empty => write!(f, "."),
            Space::Wall => write!(f, "#"),
        }
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
