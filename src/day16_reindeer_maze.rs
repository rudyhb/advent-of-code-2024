use crate::common::models::{Direction, Grid, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use utils::a_star::{self, CurrentNodeDetails, CustomNode, Node, Options, Successor};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    solve(input);
}

fn solve(input: &str) {
    let game: Game = input.parse().unwrap();

    let start = Position {
        position: game.start.clone(),
        direction: Direction::Right,
    };
    let end = Position {
        position: game.end.clone(),
        direction: Direction::Right,
    };
    let field = &game.field;

    let options = Options::default();

    let get_successors = |position: &Position| {
        let mut successors = vec![
            Successor::new(position.turned_clockwise(), 1_000),
            Successor::new(position.turned_anticlockwise(), 1_000),
        ];
        if let Some(pos) =
            field
                .0
                .try_move_if(&position.position, position.direction, |_, space| {
                    matches!(space, Space::Empty)
                })
        {
            successors.push(Successor::new(
                Position {
                    position: pos,
                    direction: position.direction,
                },
                1,
            ));
        }

        successors
    };

    fn distance_function(node_details: CurrentNodeDetails<Position, usize>) -> usize {
        node_details
            .current_node
            .position
            .manhattan_distance(&node_details.target_node.position)
    }

    let result = a_star::a_star_search(
        start.clone(),
        &end,
        get_successors,
        distance_function,
        |left, right| left.position == right.position,
        Some(&options),
    )
    .unwrap();

    let mut game = game;
    println!("shortest path len:{}", result.shortest_path.len());
    game.set_shortest_path(result.shortest_path.into_iter());
    println!("shortest path:{}", game);

    let shortest_path_cost = result.shortest_path_cost;
    println!("shortest path cost: {}", shortest_path_cost);

    let game: Game = input.parse().unwrap();

    let start = PositionWithHistory {
        position: Position {
            position: game.start.clone(),
            direction: Direction::Right,
        },
        history: 0,
    };
    let end = PositionWithHistory {
        position: Position {
            position: game.end.clone(),
            direction: Direction::Right,
        },
        history: 0,
    };
    let field = &game.field;

    let options = Options::default();

    let get_successors = |position: &PositionWithHistory| {
        let mut successors = vec![
            Successor::new(position.turned_clockwise(), 1_000),
            Successor::new(position.turned_anticlockwise(), 1_000),
        ];
        if let Some(pos) = field.0.try_move_if(
            &position.position.position,
            position.position.direction,
            |_, space| matches!(space, Space::Empty),
        ) {
            successors.push(Successor::new(position.with_new_position(pos), 1));
        }

        successors
    };

    fn distance_function_2(node_details: CurrentNodeDetails<PositionWithHistory, usize>) -> usize {
        node_details
            .current_node
            .position
            .position
            .manhattan_distance(&node_details.target_node.position.position)
    }

    let all_results = a_star::a_star_search_all_with_max_score(
        shortest_path_cost,
        start,
        &end,
        get_successors,
        distance_function_2,
        |left, right| left.position.position == right.position.position,
        Some(&options),
    )
    .unwrap();
    println!("all shortest paths len: {}", all_results.len());

    let tiles_in_path: HashSet<Point<usize>> = all_results
        .iter()
        .flat_map(|result| result.shortest_path.iter())
        .map(|position| &position.position.position)
        .cloned()
        .collect();

    let game = GameWithAllResults::new(&game, &tiles_in_path);
    println!("all tiles in all shortest paths:{}", game);

    println!("all tiles in all shortest paths:{}", tiles_in_path.len());
}

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
struct Position {
    position: Point<usize>,
    direction: Direction,
}

#[derive(Clone, Debug)]
struct PositionWithHistory {
    position: Position,
    history: u64,
}

impl Position {
    pub fn turned_clockwise(&self) -> Self {
        Self {
            position: self.position.clone(),
            direction: self.direction.turn_right(),
        }
    }
    pub fn turned_anticlockwise(&self) -> Self {
        Self {
            position: self.position.clone(),
            direction: self.direction.turn_left(),
        }
    }
}

impl PositionWithHistory {
    pub fn turned_clockwise(&self) -> Self {
        Self {
            position: self.position.turned_clockwise(),
            history: self.get_node_id(),
        }
    }
    pub fn turned_anticlockwise(&self) -> Self {
        Self {
            position: self.position.turned_anticlockwise(),
            history: self.get_node_id(),
        }
    }
    pub fn with_new_position(&self, position: Point<usize>) -> Self {
        Self {
            position: Position {
                position,
                direction: self.position.direction,
            },
            history: self.get_node_id(),
        }
    }
}

impl Node for Position {}

impl CustomNode for PositionWithHistory {
    const NODE_ID_AND_POSITION_HASH_SAME: bool = false;

    fn get_node_id(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.position.hash(&mut hasher);
        self.history.hash(&mut hasher);
        hasher.finish()
    }

    fn get_position_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.position.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Copy, Clone, Debug)]
enum Space {
    Empty,
    Wall,
}

impl Default for Space {
    fn default() -> Self {
        Self::Empty
    }
}

struct Field(Grid<Space>);

struct Game {
    field: Field,
    start: Point<usize>,
    end: Point<usize>,
    shortest_path: HashMap<Point<usize>, Direction>,
}

impl Game {
    pub fn set_shortest_path(&mut self, shortest_path: impl Iterator<Item = Position>) {
        self.shortest_path = shortest_path
            .into_iter()
            .map(|p| (p.position, p.direction))
            .collect();
    }
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size_y = s.lines().count();
        let size_x = s.lines().next().context("empty game map")?.chars().count();
        let mut grid = Grid::new(size_x, size_y);
        let mut start: Option<Point<usize>> = None;
        let mut end: Option<Point<usize>> = None;

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let point = Point { x, y };
                match c {
                    '#' => grid.set(&point, Space::Wall),
                    '.' => {}
                    'S' => {
                        start = Some(point);
                    }
                    'E' => {
                        end = Some(point);
                    }
                    other => return Err(anyhow::anyhow!("invalid character '{}'", other)),
                }
            }
        }

        Ok(Self {
            field: Field(grid),
            start: start.context("no start position found")?,
            end: end.context("no end position found")?,
            shortest_path: Default::default(),
        })
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.field.0.len_y() {
            writeln!(f)?;
            for x in 0..self.field.0.len_x() {
                let point = Point { x, y };
                if self.start == point {
                    write!(f, "S")?;
                } else if self.end == point {
                    write!(f, "E")?;
                } else if let Some(direction) = self.shortest_path.get(&point) {
                    write!(f, "{}", direction)?;
                } else {
                    match self.field.0.get(&point).unwrap() {
                        Space::Empty => {
                            write!(f, ".")?;
                        }
                        Space::Wall => {
                            write!(f, "#")?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

struct GameWithAllResults<'a>(&'a Game, &'a HashSet<Point<usize>>);

impl<'a> GameWithAllResults<'a> {
    pub fn new(game: &'a Game, all_results: &'a HashSet<Point<usize>>) -> Self {
        Self(game, all_results)
    }
}

impl Display for GameWithAllResults<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.0.field.0.len_y() {
            writeln!(f)?;
            for x in 0..self.0.field.0.len_x() {
                let point = Point { x, y };
                if self.1.contains(&point) {
                    write!(f, "O")?;
                } else {
                    match self.0.field.0.get(&point).unwrap() {
                        Space::Empty => {
                            write!(f, ".")?;
                        }
                        Space::Wall => {
                            write!(f, "#")?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "###############
#.......#....E#
#.#.###.#.###.#
#.....#.#...#.#
#.###.#####.#.#
#.#.#.......#.#
#.#.#####.###.#
#...........#.#
###.#.#####.#.#
#...#.....#.#.#
#.#.#.###.#.#.#
#.....#...#.#.#
#.###.#.#.#.#.#
#S..#.....#...#
###############",
        "#################
#...#...#...#..E#
#.#.#.#.#.#.#.#.#
#.#.#.#...#...#.#
#.#.#.#.###.#.#.#
#...#.#.#.....#.#
#.#.#.#.#.#####.#
#.#...#.#.#.....#
#.#.#####.#.###.#
#.#.#.......#...#
#.#.###.#####.###
#.#.#...#.....#.#
#.#.#.#####.###.#
#.#.#.........#.#
#.#.#.#########.#
#S#.............#
#################",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
