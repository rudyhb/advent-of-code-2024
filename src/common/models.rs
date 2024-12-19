use std::fmt::{Debug, Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, Sub};
use std::ops::{Div, Mul, Neg};

pub trait Numeric:
    Add<Output = Self>
    + Sum
    + Mul<Output = Self>
    + Div<Output = Self>
    + Invertible
    + Sub<Output = Self>
    + Default
    + Copy
    + PartialOrd
    + Debug
    + Default
{
}
impl<T> Numeric for T where
    T: Add<Output = T>
        + Sum
        + Sub<Output = T>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Invertible
        + Default
        + Copy
        + PartialOrd
        + Debug
        + Default
{
}

pub trait NumericNeg: Numeric + Neg<Output = Self> {}
impl<T> NumericNeg for T where T: Numeric + Neg<Output = Self> {}

#[derive(Clone, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub struct Point<T: Numeric> {
    pub x: T,
    pub y: T,
}

impl Point<usize> {
    pub fn manhattan_distance(&self, other: &Self) -> usize {
        let x = if self.x > other.x {
            self.x - other.x
        } else {
            other.x - self.x
        };
        let y = if self.y > other.y {
            self.y - other.y
        } else {
            other.y - self.y
        };
        x + y
    }
    pub fn move_to(&self, direction: Direction) -> Option<Self> {
        match direction {
            Direction::Up => {
                if self.y > 0 {
                    Some(Self {
                        x: self.x,
                        y: self.y - 1,
                    })
                } else {
                    None
                }
            }
            Direction::Down => Some(Self {
                x: self.x,
                y: self.y + 1,
            }),
            Direction::Left => {
                if self.x > 0 {
                    Some(Self {
                        x: self.x - 1,
                        y: self.y,
                    })
                } else {
                    None
                }
            }
            Direction::Right => Some(Self {
                x: self.x + 1,
                y: self.y,
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Grid<T> {
    map: Box<[Box<[T]>]>,
    size_x: usize,
    size_y: usize,
}

impl<T> Grid<T> {
    #[allow(dead_code)]
    pub fn new(size_x: usize, size_y: usize) -> Self
    where
        T: Default,
    {
        Self {
            map: (0..size_y)
                .map(|_| (0..size_x).map(|_| T::default()).collect())
                .collect(),
            size_x,
            size_y,
        }
    }
    pub fn iter(&self) -> GridIterator<T> {
        GridIterator::new(self)
    }
    pub fn eight_way_neighbors(&self, point: &Point<usize>) -> Vec<Point<usize>> {
        let mut neighbors = Vec::new();
        for x in point.x.saturating_sub(1)..(point.x + 2).min(self.size_x) {
            for y in point.y.saturating_sub(1)..(point.y + 2).min(self.size_y) {
                if x != point.x || y != point.y {
                    neighbors.push(Point { x, y });
                }
            }
        }
        neighbors
    }
    pub fn four_way_neighbors(&self, point: &Point<usize>) -> Vec<Point<usize>> {
        let mut neighbors = Vec::new();
        if point.x > 0 {
            neighbors.push(Point {
                x: point.x - 1,
                y: point.y,
            });
        }
        if point.x < self.size_x - 1 {
            neighbors.push(Point {
                x: point.x + 1,
                y: point.y,
            });
        }
        if point.y > 0 {
            neighbors.push(Point {
                x: point.x,
                y: point.y - 1,
            });
        }
        if point.y < self.size_y - 1 {
            neighbors.push(Point {
                x: point.x,
                y: point.y + 1,
            });
        }
        neighbors
    }
    pub fn try_move_if(
        &self,
        point: &Point<usize>,
        direction: Direction,
        condition: impl FnOnce(&Point<usize>, &T) -> bool,
    ) -> Option<Point<usize>> {
        point.move_to(direction).and_then(|point| {
            self.get(&point).and_then(|value| {
                if condition(&point, value) {
                    Some(point)
                } else {
                    None
                }
            })
        })
    }
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: Iterator,
        I::Item: IntoIterator<Item = T>,
    {
        let map: Box<[Box<[T]>]> = iter
            .into_iter()
            .map(|row| row.into_iter().collect())
            .collect();
        let size_y = map.len();
        let size_x = map.first().map_or(0, |row| row.len());
        assert!(
            map.iter().all(|row| row.len() == size_x),
            "All rows must be the same length"
        );
        Self {
            map,
            size_x,
            size_y,
        }
    }
    pub fn try_from_iter<I, E>(iter: I) -> Result<Self, E>
    where
        I: Iterator,
        I::Item: IntoIterator<Item = Result<T, E>>,
    {
        let map: Box<[Box<[T]>]> = iter
            .into_iter()
            .map(|row| row.into_iter().collect::<Result<_, E>>())
            .collect::<Result<_, E>>()?;
        let size_y = map.len();
        let size_x = map.first().map_or(0, |row| row.len());
        assert!(
            map.iter().all(|row| row.len() == size_x),
            "All rows must be the same length"
        );
        Ok(Self {
            map,
            size_x,
            size_y,
        })
    }
    pub fn len_x(&self) -> usize {
        self.size_x
    }
    pub fn len_y(&self) -> usize {
        self.size_y
    }
    pub fn get(&self, point: &Point<usize>) -> Option<&T> {
        if point.x < self.size_x && point.y < self.size_y {
            Some(&self.map[point.y][point.x])
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub fn get_mut(&mut self, point: &Point<usize>) -> Option<&mut T> {
        if point.x < self.size_x && point.y < self.size_y {
            Some(&mut self.map[point.y][point.x])
        } else {
            None
        }
    }
    pub fn set(&mut self, point: &Point<usize>, value: T) {
        self.map[point.y][point.x] = value;
    }
}

pub struct GridIterator<'a, T> {
    grid: &'a Grid<T>,
    x: usize,
    y: usize,
}

impl<'a, T> GridIterator<'a, T> {
    fn new(grid: &'a Grid<T>) -> Self {
        Self { grid, x: 0, y: 0 }
    }
    fn next_point(&mut self) {
        if self.x < self.grid.len_x() - 1 {
            self.x += 1;
        } else if self.y < self.grid.len_y() - 1 {
            self.x = 0;
            self.y += 1;
        } else {
            self.x = self.grid.len_x();
            self.y = self.grid.len_y();
        }
    }
}

impl<'a, T> Iterator for GridIterator<'a, T> {
    type Item = (Point<usize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let point = Point {
            x: self.x,
            y: self.y,
        };
        if self.x < self.grid.len_x() && self.y < self.grid.len_y() {
            self.next_point();
            let value = self.grid.get(&point).unwrap();
            Some((point, value))
        } else {
            None
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.grid.len_x() * self.grid.len_y()
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn turn_right(&self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }
    pub fn turn_left(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }
}

impl<T: Numeric> Add for &Point<T> {
    type Output = Point<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Point<usize> {
    pub fn try_sub(&self, rhs: Self) -> Option<Self> {
        if rhs.x > self.x || rhs.y > self.y {
            None
        } else {
            Some(Point {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            })
        }
    }
}
impl<T: Numeric> Sub for &Point<T> {
    type Output = Point<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Numeric + Display> Display for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[allow(dead_code)]
pub trait MultiplyByI32 {
    type Output;
    fn multiply_by_i32(self, rhs: i32) -> Self::Output;
}

impl<T> MultiplyByI32 for &Point<T>
where
    T: Numeric + Mul<i32, Output = T>,
{
    type Output = Point<T>;
    fn multiply_by_i32(self, rhs: i32) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub trait MultiplyByI64 {
    type Output;
    fn multiply_by_i64(self, rhs: i64) -> Self::Output;
}

impl<T> MultiplyByI64 for &Point<T>
where
    T: Numeric + Mul<i64, Output = T>,
{
    type Output = Point<T>;
    fn multiply_by_i64(self, rhs: i64) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

pub trait Invertible {
    fn invert(self) -> Self;
}

impl Invertible for i32 {
    fn invert(self) -> Self {
        1 / self
    }
}

impl Invertible for i64 {
    fn invert(self) -> Self {
        1 / self
    }
}

impl Invertible for usize {
    fn invert(self) -> Self {
        1 / self
    }
}

impl Invertible for f32 {
    fn invert(self) -> Self {
        1.0 / self
    }
}

impl Invertible for f64 {
    fn invert(self) -> Self {
        1.0 / self
    }
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'U' | '^' => Ok(Direction::Up),
            'D' | 'v' => Ok(Direction::Down),
            'L' | '<' => Ok(Direction::Left),
            'R' | '>' => Ok(Direction::Right),
            _ => Err(()),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => {
                write!(f, "^")
            }
            Direction::Down => {
                write!(f, "v")
            }
            Direction::Left => {
                write!(f, "<")
            }
            Direction::Right => {
                write!(f, ">")
            }
        }
    }
}
