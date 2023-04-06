use std::{error::Error, fs::File, collections::HashMap};

use osmpbfreader::OsmObj;
use parser::OsmId;

mod graph;
mod parser;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Gather all OpenStreetMap objects into a HashMap for later processing
    let mut objs: HashMap<OsmId, OsmObj> = parser::map_from_pbf(
        "resources/dortmund_sued.osm.pbf"
        // "resources/linnich.osm.pbf"
    );
    
    parser::weave(&mut objs);
    
    
    Ok(())
}

/// Runtime configuration
pub struct Config {
    
}