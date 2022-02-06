#![allow(dead_code)]
#![allow(unused_variables)]

mod lib;

use lib::{model, solver, visualizer};
use macroquad::prelude::*;
use std::{error::Error, fs};
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

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Options::from_args();

    let input_str = fs::read_to_string(opt.input)?;
    let input_yaml = YamlLoader::load_from_str(&input_str)?;
    if let Some(doc) = input_yaml.first() {
        let doc = model::Input::from(doc)?;
        if let Some(solutions) = solver::compute(&doc, opt.attempts, opt.count) {
            if !solutions.is_empty() {
                visualizer::show(&solutions).await;
            }
        }
    }

    Ok(())
}
