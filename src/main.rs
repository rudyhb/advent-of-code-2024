use std::borrow::Cow;
use utils::timer::Timer;

mod common;
mod day01_historian_hysteria;

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
        //std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
    let _timer = Timer::start(|elapsed| println!("main took {} ms", elapsed.as_millis()));

    let days = days();

    let day: usize = if let Some(arg1) = std::env::args().nth(1) {
        arg1.parse().expect("Failed to parse day number")
    } else {
        days.len()
    };

    let input = || {
        std::fs::read_to_string(format!("input/{:02}.txt", day))
            .expect(&format!("Failed to read input file input/{:02}.txt", day))
            .into()
    };
    let run = days[day - 1];

    println!("Running day {}\n", day);
    run(&input);
}

fn days() -> &'static [fn(&dyn Fn() -> Cow<'static, str>)] {
    &[
        day01_historian_hysteria::run,
    ]
}
