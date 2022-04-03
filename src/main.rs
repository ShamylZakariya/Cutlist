#![allow(dead_code)]
#![allow(unused_variables)]

mod lib;

use lib::{model, solver, visualizer};
use macroquad::prelude::*;
use std::{error::Error, fs, time::Instant};
use structopt::StructOpt;
use yaml_rust::YamlLoader;

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(short, long, default_value = "input.yaml")]
    pub input: String,

    #[structopt(short, long, default_value = "4096")]
    pub attempts: usize,

    #[structopt(short, long, default_value = "16")]
    pub count: usize,

    #[structopt(short, long)]
    pub print: bool,
}

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Cutlist"),
        window_width: 768,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

// given array of solutions, sorted from best to worst, returns tuple of (best, worst, median) score
fn scoring_stats(solutions: &[Vec<solver::Board>]) -> (f32, f32, f32) {
    let solution_scores = solutions
        .iter()
        .map(|solution| solver::score(solution))
        .collect::<Vec<_>>();

    match solution_scores.len() {
        0 => (0_f32, 0_f32, 0_f32),
        _ => {
            let best_score = solution_scores[0];
            let worst_score = solution_scores[solution_scores.len() - 1];

            let median_score = if solution_scores.len() % 2 == 1 {
                solution_scores[solution_scores.len() / 2]
            } else {
                (solution_scores[solution_scores.len() / 2]
                    + solution_scores[1 + (solution_scores.len() / 2)])
                    / 2_f32
            };

            (best_score, worst_score, median_score)
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Options::from_args();

    let input_str = fs::read_to_string(opt.input)?;
    let input_yaml = YamlLoader::load_from_str(&input_str)?;
    if let Some(doc) = input_yaml.first() {
        let doc = model::Input::from(doc)?;
        let start_time = Instant::now();
        let solutions = solver::compute(&doc, opt.attempts, opt.count);
        let elapsed_time = start_time.elapsed();

        if let Some(solutions) = solutions {
            if !solutions.is_empty() {
                let (best, worst, median) = scoring_stats(&solutions);

                println!(
                    "Solving {} attempts took {:?}\nScoring best: {} worst: {} median: {}",
                    opt.attempts, elapsed_time, best, worst, median
                );

                visualizer::show(&solutions, opt.print).await;
            }
        }
    }

    Ok(())
}
