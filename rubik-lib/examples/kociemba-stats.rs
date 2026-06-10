use std::time::Duration;

use plotters::prelude::*;
use rubik_lib::{
    core::{scramble::scramble, state::State},
    solve::kociemba::{self},
};

fn check_sol(mut state: State, moves: &[u8]) {
    for &mv in moves {
        state = state * State::BASIC_MOVES[mv as usize];
    }
    assert_eq!(state, State::ID);
}

fn avg_sol_length(solver: &mut kociemba::Solver, timeout: Duration, n: usize) -> f64 {
    let mut total_length = 0;
    for _ in 0..n {
        let (_, state) = scramble(100);
        let sol = solver.solve_timeout(&state, timeout);
        check_sol(state, &sol);
        total_length += sol.len();
    }
    total_length as f64 / n as f64
}

fn main() {
    let min_time: f64 = 1.0;
    let max_time: f64 = 1000.0;
    let num_points = 50;
    let meas_per_point = 50;

    let mut solver = kociemba::Solver::from_folder("data/kociemba").expect("failed to init solver");
    let mut series = vec![];
    for i in 0..=num_points {
        let log_min = min_time.ln();
        let log_max = max_time.ln();
        let t = i as f64 / num_points as f64;
        let log = (1.0 - t) * log_min + t * log_max;
        let ms = log.exp();
        let len = avg_sol_length(&mut solver, Duration::from_secs_f64(ms / 1000.0), meas_per_point);
        println!("{:.1}ms: {:.1}", ms, len);
        series.push((ms, len));
    }

    let root = SVGBackend::new("data/kociemba-stats.svg", (640, 480)).into_drawing_area();
    let _ = root.fill(&WHITE);
    let mut chart = ChartBuilder::on(&root)
        .caption("Kociemba solution length", ("sans-serif", 20).into_font())
        .margin(25)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(min_time..max_time, 15.0..25.0)
        .unwrap();

    let _ = chart
        .configure_mesh()
        .x_desc("Solve time [ms]")
        .y_desc("Average solution length")
        .draw();
    let _ = chart.draw_series(LineSeries::new(series, &RED));
    let _ = root.present();
}
