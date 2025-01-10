#![feature(linked_list_cursors)]

use utils::timer::Timer;

mod common;
mod day01_historian_hysteria;
mod day02_red_nosed_reports;
mod day03_mull_it_over;
mod day04_ceres_search;
mod day05_print_queue;
mod day06_guard_gallivant;
mod day07_bridge_repair;
mod day08_resonant_collinearity;
mod day09_disk_fragmenter;
mod day10_hoof_it;
mod day11_plutonian_pebbles;
mod day12_garden_groups;
mod day13_claw_contraption;
mod day14_restroom_redoubt;
mod day15_warehouse_woes;
mod day16_reindeer_maze;
mod day17_chronospatial_computer;
mod day18_ram_run;
mod day19_linen_layout;
mod day20_race_condition;
mod day21_keypad_conundrum;
mod day22_monkey_market;
mod day23_lan_party;
mod day24_crossed_wires;

fn main() {
    let mut context = common::Context::default();

    if let Ok(testing) = std::env::var("APP_TESTING") {
        if let Ok(testing) = testing.parse() {
            context.set_testing(testing);
        }
    }
    //context.set_testing(0);

    if std::env::var("RUST_LOG").is_err() {
        if context.is_testing() {
            std::env::set_var("RUST_LOG", "trace");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();
    let _timer = Timer::start(|elapsed| println!("main took {} ms", elapsed.as_millis()));

    let days = days();

    let day: usize = if let Some(arg1) = std::env::args().nth(1) {
        arg1.parse().expect("Failed to parse day number")
    } else {
        days.len()
    };

    context.set_text_input(Box::new(move || {
        std::fs::read_to_string(format!("input/{:02}.txt", day))
            .unwrap_or_else(|_| panic!("Failed to read input file input/{:02}.txt", day))
            .into()
    }));
    let run = days[day - 1];

    println!("Running day {}\n", day);
    run(&mut context);
}

fn days() -> &'static [fn(&mut common::Context)] {
    &[
        day01_historian_hysteria::run,
        day02_red_nosed_reports::run,
        day03_mull_it_over::run,
        day04_ceres_search::run,
        day05_print_queue::run,
        day06_guard_gallivant::run,
        day07_bridge_repair::run,
        day08_resonant_collinearity::run,
        day09_disk_fragmenter::run,
        day10_hoof_it::run,
        day11_plutonian_pebbles::run,
        day12_garden_groups::run,
        day13_claw_contraption::run,
        day14_restroom_redoubt::run,
        day15_warehouse_woes::run,
        day16_reindeer_maze::run,
        day17_chronospatial_computer::run,
        day18_ram_run::run,
        day19_linen_layout::run,
        day20_race_condition::run,
        day21_keypad_conundrum::run,
        day22_monkey_market::run,
        day23_lan_party::run,
        day24_crossed_wires::run,
    ]
}
