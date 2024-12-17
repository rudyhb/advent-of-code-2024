use crate::common::models::{Direction, Grid, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    solve(input, false);

    solve(input, true);
}

fn solve(input: &str, is_wide: bool) {
    let mut parts = input.split("\n\n");
    let mut map: Map = if is_wide {
        parts.next().unwrap().parse::<WideMap>().unwrap().0
    } else {
        parts.next().unwrap().parse().unwrap()
    };

    log::debug!("initial state:{}", map);
    for c in parts.next().unwrap().chars() {
        if c.is_whitespace() {
            continue;
        }
        map.next(
            c.try_into()
                .ok()
                .with_context(|| format!("invalid direction: '{}'", c))
                .unwrap(),
        );
        log::trace!("Move {}:{}", c, map);
    }
    log::debug!("end state:{}", map);

    println!("sum gps: {}", map.sum_gps());
}

struct Map {
    grid: Grid<Space>,
    robot: Point<usize>,
}

#[derive(Default)]
struct SaveStage(Option<Grid<Space>>);
impl SaveStage {
    pub fn save(&mut self, source: &Map) {
        self.0 = Some(source.grid.clone())
    }
    pub fn load(&mut self, dest: &mut Map) {
        let data = std::mem::take(&mut self.0);
        dest.grid = data.unwrap();
    }
}

impl Map {
    pub fn next(&mut self, direction: Direction) {
        let point = self.robot.clone();
        self.try_move(&point, direction);
    }
    fn try_move(&mut self, point: &Point<usize>, direction: Direction) -> bool {
        let mut data = SaveStage::default();

        let point_next = point
            .move_to(direction)
            .expect("weird, the walls disappeared?");
        if match *self.grid.get(&point_next).expect("where did the grid go?") {
            Space::Empty => true,
            Space::Wall => false,
            Space::Box => self.try_move(&point_next, direction),
            Space::Robot => {
                unreachable!()
            }
            Space::WideBox => {
                data.save(&self);
                let point_next_2 = point_next
                    .move_to(Direction::Right)
                    .expect("WideBox doesn't have a right side");

                if self.try_move(&point_next_2, direction) && self.try_move(&point_next, direction)
                {
                    // Order Important!
                    true
                } else {
                    data.load(self);
                    false
                }
            }
            Space::WideBoxEnd => {
                data.save(&self);
                let point_next_2 = point_next
                    .move_to(Direction::Left)
                    .expect("WideBox doesn't have a left side");
                if self.try_move(&point_next_2, direction) && self.try_move(&point_next, direction)
                {
                    // Order Important!
                    true
                } else {
                    data.load(self);
                    false
                }
            }
        } {
            let value_next = *self
                .grid
                .get(&point_next)
                .expect("where did the grid go again?");
            let value = *self
                .grid
                .get(&point)
                .expect("where is the grid coming from?");
            if let Space::Robot = value {
                self.robot = point_next.clone();
            }
            log::trace!(
                "swapping {} {} and {} {}",
                point,
                value,
                point_next,
                value_next
            );
            self.grid.set(&point, value_next);
            self.grid.set(&point_next, value);
            true
        } else {
            false
        }
    }
    pub fn sum_gps(&self) -> usize {
        self.grid
            .iter()
            .filter_map(|(point, value)| match value {
                Space::Box | Space::WideBox => Some(point.y * 100 + point.x),
                _ => None,
            })
            .sum()
    }
}

#[derive(Copy, Clone)]
enum Space {
    Empty,
    Wall,
    Box,
    Robot,
    WideBox,
    WideBoxEnd,
}

impl Default for Space {
    fn default() -> Self {
        Self::Empty
    }
}

impl FromStr for Map {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size_y = s.lines().count();
        let first_line = s.lines().next().context("empty map")?;
        let size_x = first_line.chars().count();
        if first_line.chars().any(|c| c != '#') {
            return Err(anyhow::anyhow!("first line should be all walls"));
        }

        let mut grid: Grid<Space> = Grid::new(size_x, size_y);
        let mut robot: Option<Point<usize>> = None;

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let point = Point { x, y };
                match c {
                    '.' => {}
                    '#' => {
                        grid.set(&point, Space::Wall);
                    }
                    'O' => {
                        grid.set(&point, Space::Box);
                    }
                    '@' => {
                        grid.set(&point, Space::Robot);
                        robot = Some(point);
                    }
                    _ => return Err(anyhow::anyhow!("invalid character: {}", c)),
                }
            }
        }

        Ok(Self {
            grid,
            robot: robot.context("robot not found")?,
        })
    }
}

struct WideMap(Map);

impl FromStr for WideMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size_y = s.lines().count();
        let first_line = s.lines().next().context("empty map")?;
        let size_x = first_line.chars().count() * 2;
        if first_line.chars().any(|c| c != '#') {
            return Err(anyhow::anyhow!("first line should be all walls"));
        }

        let mut grid: Grid<Space> = Grid::new(size_x, size_y);
        let mut robot: Option<Point<usize>> = None;

        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let point = Point { x: 2 * x, y };
                let point2 = Point { x: 2 * x + 1, y };
                match c {
                    '.' => {}
                    '#' => {
                        grid.set(&point, Space::Wall);
                        grid.set(&point2, Space::Wall);
                    }
                    'O' => {
                        grid.set(&point, Space::WideBox);
                        grid.set(&point2, Space::WideBoxEnd);
                    }
                    '@' => {
                        grid.set(&point, Space::Robot);
                        robot = Some(point);
                    }
                    _ => return Err(anyhow::anyhow!("invalid character: {}", c)),
                }
            }
        }

        Ok(Self(Map {
            grid,
            robot: robot.context("robot not found")?,
        }))
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Empty => write!(f, "."),
            Space::Wall => write!(f, "#"),
            Space::Box => write!(f, "O"),
            Space::Robot => write!(f, "@"),
            Space::WideBox => write!(f, "["),
            Space::WideBoxEnd => write!(f, "]"),
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.grid.len_y() {
            write!(f, "\n")?;
            for x in 0..self.grid.len_x() {
                let point = Point { x, y };
                if let Some(space) = self.grid.get(&point) {
                    write!(f, "{}", space)?;
                } else {
                    write!(f, ".")?;
                }
            }
        }
        Ok(())
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########

<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^",
        "########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv<v>>v<<",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
