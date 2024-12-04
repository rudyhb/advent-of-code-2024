use crate::common::models::{Grid, Point};
use crate::common::{Context, InputProvider};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let xmas_count = find_xmas_count(input);
    println!("Xmas count: {}", xmas_count);

    let xmas_count_2 = find_xmas_count_v2(input);
    println!("Xmas count v2: {}", xmas_count_2);
}

fn find_xmas_count(input: &str) -> usize {
    let grid = parse_input(input);
    grid.iter()
        .filter(|(_, &value)| value == 'X')
        .map(|(point, _)| {
            grid.eight_way_neighbors(&point)
                .iter()
                .filter(|point_2| grid.get(point_2) == Some(&'M'))
                .filter(|point_2| {
                    let direction_x: i32 = point_2.x as i32 - point.x as i32;
                    let direction_y: i32 = point_2.y as i32 - point.y as i32;

                    if point_2.x as i32 + 2 * direction_x < 0
                        || point_2.y as i32 + 2 * direction_y < 0
                    {
                        return false;
                    }

                    let point_3 = Point {
                        x: (point_2.x as i32 + direction_x) as usize,
                        y: (point_2.y as i32 + direction_y) as usize,
                    };

                    let point_4 = Point {
                        x: (point_3.x as i32 + direction_x) as usize,
                        y: (point_3.y as i32 + direction_y) as usize,
                    };

                    if grid.get(&point_3) == Some(&'A') && grid.get(&point_4) == Some(&'S') {
                        true
                    } else {
                        false
                    }
                })
                .count()
        })
        .sum()
}
fn find_xmas_count_v2(input: &str) -> usize {
    let grid = parse_input(input);
    grid.iter()
        .filter(|(_, &value)| value == 'A')
        .filter(|(point, _)| {
            if point.x < 1
                || point.y < 1
                || point.x > grid.len_x() - 2
                || point.y > grid.len_y() - 2
            {
                return false;
            }

            let point_1 = Point {
                x: point.x - 1,
                y: point.y - 1,
            };
            let point_2 = Point {
                x: point.x - 1,
                y: point.y + 1,
            };
            let point_3 = Point {
                x: point.x + 1,
                y: point.y - 1,
            };
            let point_4 = Point {
                x: point.x + 1,
                y: point.y + 1,
            };

            let value_1 = grid.get(&point_1).unwrap();
            let value_2 = grid.get(&point_2).unwrap();
            let value_3 = grid.get(&point_3).unwrap();
            let value_4 = grid.get(&point_4).unwrap();
            let values = [value_1, value_2, value_3, value_4];

            let count_s = values.iter().filter(|&&&c| c == 'S').count();
            let count_m = values.iter().filter(|&&&c| c == 'M').count();

            if count_s != 2 || count_m != 2 {
                return false;
            }

            value_1 != value_4
        })
        .count()
}

fn parse_input(input: &str) -> Grid<char> {
    Grid::from_iter(input.lines().map(|line| line.chars()))
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
