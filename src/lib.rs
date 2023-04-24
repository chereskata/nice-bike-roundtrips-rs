use std::{error::Error};

use parser::OsmData;

// map data structure
mod graph;
// make osm.pbf files useable
mod parser;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Gather all OpenStreetMap objects into a HashMap for later processing
    let mut data: OsmData = parser::data_from_pbf(
        "resources/dortmund_sued.osm.pbf"
        // "resources/linnich.osm.pbf"
    );
    
    parser::weave(&mut data);
    
    
    Ok(())
}

/// Runtime configuration
pub struct Config {
    
}