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

fn main() {
    let mut context = common::Context::default();
    //context.set_testing(0);

    if let Ok(testing) = std::env::var("APP_TESTING") {
        if let Ok(testing) = testing.parse() {
            context.set_testing(testing);
        }
    }

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
            .expect(&format!("Failed to read input file input/{:02}.txt", day))
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
    ]
}
