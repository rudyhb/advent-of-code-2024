use crate::common::linear_algebra::{self, Matrix, Vector};
use crate::common::models::Point;
use crate::common::{Context, InputProvider};
use once_cell::sync::Lazy;
use regex::Regex;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let games: Vec<Game> = input.split("\n\n").map(|s| s.parse().unwrap()).collect();

    let solution: i64 = games
        .iter()
        .filter_map(|game| solve_game(game))
        .map(|solution| solution.x * 3 + solution.y * 1)
        .sum();
    println!("solution 1: {}", solution);

    let mut games = games;
    for game in games.iter_mut() {
        game.price.x += 10000000000000.0;
        game.price.y += 10000000000000.0;
    }
    let games = games;

    let solution: i64 = games
        .iter()
        .filter_map(|game| solve_game(game))
        .map(|solution| solution.x * 3 + solution.y * 1)
        .sum();
    println!("solution 2: {}", solution);
}

fn solve_game(game: &Game) -> Option<Point<i64>> {
    let matrix: Matrix<_, 2, 2> = [
        [game.button_a.delta.x, game.button_b.delta.x],
        [game.button_a.delta.y, game.button_b.delta.y],
    ]
    .into();
    let prize: Vector<_, 2> = [game.price.x, game.price.y].into();
    let solution = linear_algebra::solve_2x2_matrix_ax_b(&matrix, &prize);
    let mut point = Point::default();
    for (float, int) in [(solution[0], &mut point.x), (solution[1], &mut point.y)] {
        *int = float.round() as i64;
        if (float - *int as f64).abs() > 0.001 {
            log::debug!("no solution for game:\n{:?}", game);
            return None;
        }
    }
    log::debug!("solution = {} for game:\n{:?}", point, game);
    Some(point)
}

#[derive(Debug)]
struct Game {
    button_a: Button,
    button_b: Button,
    price: Point<f64>,
}

#[derive(Debug)]
struct Button {
    delta: Point<f64>,
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Result<Regex, regex::Error>> = Lazy::new(|| {
            Regex::new(
                r"Button A: X(?<ax>[-+]\d+), Y(?<ay>[-+]\d+)
Button B: X(?<bx>[-+]\d+), Y(?<by>[-+]\d+)
Prize: X=(?<px>[-+]?\d+), Y=(?<py>[-+]?\d+)",
            )
        });
        match &*RE {
            Err(e) => Err(anyhow::anyhow!(
                "failed to compile regex: {}",
                e.to_string().replace("\n", "")
            )),
            Ok(re) => {
                if let Some(caps) = re.captures(s) {
                    Ok(Game {
                        button_a: Button {
                            delta: Point {
                                x: caps["ax"].parse()?,
                                y: caps["ay"].parse()?,
                            },
                        },
                        button_b: Button {
                            delta: Point {
                                x: caps["bx"].parse()?,
                                y: caps["by"].parse()?,
                            },
                        },
                        price: Point {
                            x: caps["px"].parse()?,
                            y: caps["py"].parse()?,
                        },
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "Input string does not match the expected format."
                    ))
                }
            }
        }
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["Button A: X+94, Y+34
Button B: X+22, Y+67
Prize: X=8400, Y=5400

Button A: X+26, Y+66
Button B: X+67, Y+21
Prize: X=12748, Y=12176

Button A: X+17, Y+86
Button B: X+84, Y+37
Prize: X=7870, Y=6450

Button A: X+69, Y+23
Button B: X+27, Y+71
Prize: X=18641, Y=10279"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
