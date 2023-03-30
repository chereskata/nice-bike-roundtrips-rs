use std::{error::Error, fs::File};

mod graph;
mod parser;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let f = File::open("resources/dortmund_sued.osm.pbf")
        .expect("Could not find .pbf file");
    
    parser::weave(f);
    
    
    Ok(())
}

/// Runtime configuration
pub struct Config {
    
}