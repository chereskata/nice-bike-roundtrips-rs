use std::{fs::File, io::BufReader, process::exit};
use core::panic;
use osmpbfreader::{OsmPbfReader, OsmObj};

mod data;
mod network;
mod surrounding;

pub use crate::parser::data::*;
pub use crate::parser::network::*;
pub use crate::parser::surrounding::*;

/// Returns a container of every Node, Way and Realation in an pbf file.
/// note: could be optimized to return just a somewhat useful subset to reduce
/// memory footprint
pub fn data_from_pbf(path: &str) -> OsmData {
    let pbf = File::open(path)
            .expect("Could not find .pbf file");
    
    let mut buf = OsmPbfReader::new(BufReader::new(pbf));


    // 2^11 = 2048 => The highest 11 bits can be used to give each way chunk an
    // unique id, while retaining the OSM WayId to be used for simple tag lookups

    // note: highest NodeId here: https://textual.ru/64/
    // note: ways are allowed to have up to 2000 nodes: https://wiki.openstreetmap.org/wiki/Way

    // Highest 11 bits shall be unuse for identifying way chunks in the Graph
    const MAX_WAY_ID: u64 = u64::pow(2, 53) - 1;
    
    let mut data = OsmData::new();
    for chunk in buf.iter() {
        match chunk {
            Ok(obj) => 
                match obj {
                    OsmObj::Node(n) => {
                        data.nodes.insert(n.id.0.unsigned_abs(), n);
                    },
                    OsmObj::Way(w) => {
                        let way_id = w.id.0.unsigned_abs();
                        if MAX_WAY_ID < way_id { panic!("WayId {way_id} is higher than 2^53-1"); } 
                        data.ways.insert(way_id, w);
                    },
                    OsmObj::Relation(r) => {
                        data.relations.insert(r.id.0.unsigned_abs(), r);
                    },
                },
            Err(e) => eprintln!("{:?}", e),
        }
    }

    data
}

/// Print the OsmObj
pub fn print_object(obj: &OsmObj) {
    match obj {
        OsmObj::Node(node) => {
            println!("Node with ID:{} has following tags:", node.id.0);
            for tag in node.tags.iter() {
                println!("    tag:{} => value:{}", tag.0, tag.1);
            }
            println!("Node has following lat:{}, lon:{}", node.lat(), node.lon());
        },
        OsmObj::Way(way) => {
            println!("Way with ID:{} has following tags:", way.id.0);
            for tag in way.tags.iter() {
                println!("    tag:{} => value:{}", tag.0, tag.1); 
            }
        
            println!("It contains the following nodes ({}):", way.nodes.len());
            for node_id in way.nodes.iter() {
                println!("Point with node id: {}", node_id.0);   
            }             
        },
        OsmObj::Relation(_) => {
            eprintln!("Relation found, exiting ...");
            exit(1);
        },
    }
}
