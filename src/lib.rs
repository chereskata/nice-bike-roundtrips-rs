use core::panic;
use std::{error::Error, fs::File, io::{BufReader, Read}, println, path::Path};

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
    let start_point = Point::new(        
        config.start_lon.clone(),
        config.start_lat.clone()
    );


    // real_travel_distance is in meters, so convert the expected distance to meters
    // and add 10% to it
    let expected_travel_distance = config.distance as f64 * 1100.0;
    let mut real_travel_distance = f64::MAX;
    let mut gpx: Option<gpx::Gpx> = None;
    
    while real_travel_distance > expected_travel_distance {
        let interesting_points = parser::interesting_surrounding(&data, &start_point, &config.distance);
        let mut visit = router::nearest_graph_nodes(&graph, &interesting_points);

        let start = router::closest_point(&graph, &start_point);
        let route: Vec<NodeId> = router::unoptimized(&graph, &mut visit, &start);

        gpx = Some(router::postprocessor::intersections_to_gpx(&graph, &route));
        real_travel_distance = geo::algorithm::HaversineLength::haversine_length(
            &mut gpx.clone().unwrap().routes.first().unwrap().linestring()
        );
    }

    
    let gpx_path = Path::new("/tmp/result.gpx");
    let gpx_file = match File::create(&gpx_path) {
        Err(why) => panic!("couldn't create path: {}", why),
        Ok(file) => file,
    };

    gpx::write(&gpx.unwrap(), gpx_file).ok();

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