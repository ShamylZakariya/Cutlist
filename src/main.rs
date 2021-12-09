
#![allow(dead_code)]
#![allow(unused_variables)]


use std::{error::Error, fs};

use structopt::StructOpt;
use yaml_rust::YamlLoader;

mod model;
mod cutlist;

#[derive(StructOpt,Debug)]
pub struct Options {
    #[structopt(short, long, default_value = "input.yaml")]
    pub input: String,

    #[structopt(short, long)]
    pub visualize: bool,
}

fn main() -> Result<(), Box<dyn Error>>{
    let opt = Options::from_args();

    let input_str = fs::read_to_string(opt.input)?;
    let input_yaml = YamlLoader::load_from_str(&input_str)?;
    if let Some(doc) = input_yaml.first() {
        let doc = model::Input::from(doc)?;
        let cutlist = cutlist::compute(&doc, 16)?;

        if opt.visualize {
            // kick off some macroquad vis of the cutlist
        }
    }

    Ok(())
}
