
use std::{error::Error, fs};

use structopt::StructOpt;
use yaml_rust::YamlLoader;

mod specification;

#[derive(StructOpt,Debug)]
pub struct Options {
    #[structopt(short, long, default_value = "input.yaml")]
    pub input: String,
}

fn main() -> Result<(), Box<dyn Error>>{
    let opt = Options::from_args();

    let input_str = fs::read_to_string(opt.input)?;
    let input_yaml = YamlLoader::load_from_str(&input_str)?;
    if let Some(doc) = input_yaml.first() {
        let doc = specification::Input::from(doc)?;
        println!("doc:\n{:#?}", doc);
    }


    Ok(())
}
