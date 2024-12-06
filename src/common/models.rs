#[derive(Clone, Default, Hash, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
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
    pub fn eight_way_neighbors(&self, point: &Point) -> Vec<Point> {
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
    #[allow(dead_code)]
    pub fn four_way_neighbors(&self, point: &Point) -> Vec<Point> {
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
    pub fn len_x(&self) -> usize {
        self.size_x
    }
    pub fn len_y(&self) -> usize {
        self.size_y
    }
    pub fn get(&self, point: &Point) -> Option<&T> {
        if point.x < self.size_x && point.y < self.size_y {
            Some(&self.map[point.y][point.x])
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub fn get_mut(&mut self, point: &Point) -> Option<&mut T> {
        if point.x < self.size_x && point.y < self.size_y {
            Some(&mut self.map[point.y][point.x])
        } else {
            None
        }
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
    type Item = (Point, &'a T);

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
}