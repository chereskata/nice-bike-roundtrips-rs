use core::panic;
use std::{error::Error, fs::File, io::{BufReader, Read}, println};

use geo::Point;
use graph::{Graph, NodeId};
use parser::OsmData;

// map data structure
mod graph;
// make osm.pbf files useable
mod parser;
// all routing algorithms are implemented here
mod router;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Gather all OpenStreetMap objects into a HashMap for later processing
    let mut data: OsmData = parser::data_from_pbf(
        &config.pbf
    );
    
    let graph: Graph = parser::weave(&mut data);
    let start = router::closest_point(&graph, &Point::new(
        config.start_lat.clone(),
        config.start_lon.clone()
    ));

    let interesting_points = parser::interesting_points(&data);
    let mut visit = router::nearest_graph_nodes(&graph, &interesting_points);
    
    let route: Vec<NodeId> = router::unoptimized(&graph, &mut visit, &start);

    println!("{:?}", route);
    
    Ok(())
}

/// Runtime configuration
#[derive(serde::Deserialize)]
pub struct Config {
    distance: u8,
    start_lat: f64,
    start_lon: f64,
    pbf: String,
}

impl Config {
    pub fn from(f: File) -> Self {
        let mut reader = BufReader::new(f);
        let mut str = String::new();
        if let Err(e) = reader.read_to_string(&mut str) {
            panic!("{}", e);
        }
        
        match toml::from_str(&str) {
            Ok(t) => t,
            Err(e) => panic!("{}", e),
        }
    }
}