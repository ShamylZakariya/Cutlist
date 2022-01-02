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

    #[structopt(short, long)]
    pub visualize: bool,

    #[structopt(short, long, default_value = "1024")]
    pub attempts: usize,

    #[structopt(short, long, default_value = "1")]
    pub count: usize,
}

fn test_cutlist() -> Vec<solver::Board> {
    vec![solver::Board {
        length: 92f32,
        width: 8f32,
        id: "A".to_string(),
        stacks: vec![
            solver::CutStack {
                cuts: vec![
                    solver::Cut {
                        length: 18f32,
                        width: 3f32,
                        id: "Apron 1".to_string(),
                    },
                    solver::Cut {
                        length: 18f32,
                        width: 3f32,
                        id: "Apron 2".to_string(),
                    },
                ],
            },
            solver::CutStack {
                cuts: vec![
                    solver::Cut {
                        length: 9f32,
                        width: 2f32,
                        id: "Something".to_string(),
                    },
                    solver::Cut {
                        length: 10f32,
                        width: 2f32,
                        id: "Something Else".to_string(),
                    },
                    solver::Cut {
                        length: 11f32,
                        width: 2f32,
                        id: "Something Longer".to_string(),
                    },
                ],
            },
        ],
    }]
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
    // visualizer::show(&test_cutlist()).await;

    let opt = Options::from_args();

    let input_str = fs::read_to_string(opt.input)?;
    let input_yaml = YamlLoader::load_from_str(&input_str)?;
    if let Some(doc) = input_yaml.first() {
        let doc = model::Input::from(doc)?;
        if let Some(cutlist) = solver::compute(&doc, opt.attempts, opt.count) {
            //if opt.visualize {
            if let Some(cutlist) = cutlist.first() {
                // TODO: We need to pass all results to visualizer and provide a UX for paging through them
                visualizer::show(cutlist).await;
            }
            //}
        }
    }

    Ok(())
}
