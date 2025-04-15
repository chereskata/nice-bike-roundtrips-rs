use core::panic;
use std::{error::Error, fs::File, io::{BufReader, Read}, println, path::Path};

use geo::{Length, Point};
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
    let expected_travel_distance = (config.distance as f64 * 0_900.0, config.distance as f64 * 1_100.0);
    let mut real_travel_distance = f64::MAX;
    let mut gpx: Option<gpx::Gpx> = None;

    println!("exp {:?}", expected_travel_distance);
    
    while real_travel_distance < expected_travel_distance.0 ||
          real_travel_distance > expected_travel_distance.1 {
        let interesting_points = parser::interesting_surrounding(&data, &start_point, &config.distance);
        let mut visit = router::nearest_graph_nodes(&graph, &interesting_points);

        let start = router::closest_point(&graph, &start_point);
        let route: Vec<NodeId> = router::unoptimized(&graph, &mut visit, &start);

        gpx = Some(router::postprocessor::intersections_to_gpx(&graph, &route));
        let ls = gpx.clone().unwrap().routes.first().unwrap().linestring();

        // check if start and end is equal
        if ls.0.first() != ls.0.last() { continue; }

        // if more than 25% of points are visited twice, it is not a nice route
        let mut ls2 = ls.0.clone();
        ls2.dedup();
        if ls2.len() < ls.0.len() * 0.75 as usize { continue; }
        
        real_travel_distance = geo::algorithm::Haversine.length(
            &mut gpx.clone().unwrap().routes.first().unwrap().linestring()
        );
        println!("dist is {}", real_travel_distance);
    }

    
    let gpx_path = Path::new(&config.result);
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
    result: String
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