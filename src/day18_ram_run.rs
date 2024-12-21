use crate::common::models::{Grid, Point};
use crate::common::{Context, InputProvider};
use std::fmt::{Display, Formatter};
use utils::a_star::{a_star_search, ComputationResult, Options, Successor};
use utils::bisection_method;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let (initial_corruption, grid_size) = if context.is_testing() {
        (12, 7)
    } else {
        (1024, 71)
    };

    let input = context.get_input();
    let input = input.as_str();

    let corruption = parse(input);
    let mut memory_space = MemorySpace::new(Grid::new(grid_size, grid_size));

    for corruption in corruption.iter().take(initial_corruption) {
        memory_space.add_corruption(corruption);
    }
    log::debug!(
        "initial corruption after {} bytes:{}",
        initial_corruption,
        memory_space.grid
    );

    let (shortest_path, shortest_path_cost) = solve(&memory_space, None).unwrap();
    log::debug!(
        "shortest path:{}",
        memory_space
            .grid
            .display_with_overrides(shortest_path.iter().map(|p| (p, &'O')).collect())
    );
    println!("shortest path cost: {}", shortest_path_cost);

    let memory_space = MemorySpace::new(Grid::new(grid_size, grid_size));

    let options = Options::default().with_no_logs();
    let first_preventing_exit = bisection_method::find_first_true(
        |corruption_size| {
            let mut memory_space = memory_space.clone();
            for corruption in corruption.iter().take(corruption_size + 1) {
                memory_space.add_corruption(corruption);
            }
            if let Ok((path, _cost)) = solve(&memory_space, Some(&options)) {
                log::debug!(
                    "corruption index {} still has an exit:{}",
                    corruption_size,
                    memory_space
                        .grid
                        .display_with_overrides(path.iter().map(|p| (p, &'O')).collect())
                );
                false
            } else {
                log::debug!(
                    "corruption index {} has NO exit:{}",
                    corruption_size,
                    memory_space.grid
                );
                true
            }
        },
        0,
        corruption.len() - 1,
    )
    .expect("first blocking exit not found");

    println!(
        "first blocking exit is {}: {}",
        first_preventing_exit, corruption[first_preventing_exit]
    );
}

fn solve(
    memory_space: &MemorySpace,
    options: Option<&Options>,
) -> Result<(Vec<Point<usize>>, usize), anyhow::Error> {
    let start = Point::new(0, 0);
    let end = Point::new(memory_space.grid.len_x() - 1, memory_space.grid.len_y() - 1);
    let ComputationResult {
        shortest_path,
        shortest_path_cost,
    } = a_star_search(
        start,
        &end,
        |node| {
            memory_space
                .grid
                .four_way_neighbors(node)
                .into_iter()
                .filter(|next| matches!(memory_space.grid.get(&next), Some(Space::Empty)))
                .map(|next| Successor::new(next, 1))
                .collect()
        },
        |details| details.current_node.manhattan_distance(details.target_node),
        |left, right| left == right,
        options,
    )?;
    Ok((shortest_path, shortest_path_cost))
}

fn parse(input: &str) -> Vec<Point<usize>> {
    input
        .lines()
        .map(|line| {
            let mut parts = line.split(',');
            Point {
                x: parts.next().unwrap().parse().unwrap(),
                y: parts.next().unwrap().parse().unwrap(),
            }
        })
        .collect()
}

#[derive(Clone)]
struct MemorySpace {
    grid: Grid<Space>,
    fallen_bytes: usize,
}

impl MemorySpace {
    pub fn new(grid: Grid<Space>) -> Self {
        Self {
            grid,
            fallen_bytes: 0,
        }
    }
    pub fn add_corruption(&mut self, corruption: &Point<usize>) {
        self.fallen_bytes += 1;
        self.grid.set(corruption, Space::Corrupted);
    }
}

#[derive(Clone, Copy)]
enum Space {
    Empty,
    Corrupted,
}

impl Default for Space {
    fn default() -> Self {
        Self::Empty
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Empty => {
                write!(f, ".")?;
            }
            Space::Corrupted => {
                write!(f, "#")?;
            }
        }
        Ok(())
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["5,4
4,2
4,5
3,0
2,1
6,3
2,4
1,5
0,6
3,3
2,6
5,1
1,2
5,5
2,5
6,5
1,4
0,4
6,4
1,1
6,1
1,0
0,5
1,6
2,0"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
