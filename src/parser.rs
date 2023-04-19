use core::panic;
use std::{fs::File, io::BufReader, process::exit, collections::HashMap};

use osmpbfreader::{OsmPbfReader, OsmObj, Node as OsmNode, Way as OsmWay};

use crate::graph::Graph;

mod data;

pub use crate::parser::data::*;

/// Build up a Graph from the routable part of OpenStreetMap data
pub fn weave(data: &OsmData) -> Graph {
    let ways: Vec<WayId> = bikeable_ways(&data.ways);

    // nodes of the bikeable part of the street network
    // note: the value is an info, if the node is an intersection 
    let mut nodes: HashMap<NodeId, bool> = HashMap::new();

    // loop trough every way and read out his node ids
    // note: if a node is already registered in a previous iteration, multiple
    // ways cross it and it is an intersection
    for way_id in ways.iter() {
        data.ways.get(&way_id).unwrap().nodes
            .iter()
            .for_each(|node_id| {
                let node_id = node_id.0.unsigned_abs();
                if nodes.contains_key(&node_id) {
                    // node is an intersection
                    nodes.insert(node_id, true);
                } else {
                    nodes.insert(node_id, false);
                }
            });
    }

    // build up graph data
    use crate::graph::*;
    let mut graph_nodes: HashMap<NodeId, Node> = HashMap::new();
    let mut graph_edges: HashMap<EdgeId, Edge> = HashMap::new();


    // split way at every intersection
    for way_id in ways.iter() {
        let way = data.ways.get(&way_id).unwrap();
        
        // collect all nodes from the way, shall be sorted
        let nodes_of_way: Vec<NodeId> = way.nodes
            .iter()
            .map(|node_id| node_id.0.unsigned_abs())
            .collect();

        // check, if the edge is directed
        // note: could be extracted into own function
        let directed = way.tags
            .iter()
            .any(|tag| {
                if tag.0.as_str() == "oneway" {
                    match tag.1.as_str() {
                        "yes" | "1" | "true" => return true,
                        _ => return false
                    }
                }
                false
        });
        
        let mut way_chunks: Vec<Vec<NodeId>> = Vec::new();

        let mut chunk: Vec<NodeId> = Vec::new();
        for node_id in nodes_of_way {
            chunk.push(node_id.clone());
            // if node is an intersection
            if *nodes.get(&node_id).unwrap() {
                // split the way here
                way_chunks.push(chunk); // left split contains intersection
                chunk = Vec::new();
                chunk.push(node_id); // right split contains intersection
            }
        };
        
        // if a way is unreachable, not one node is a intersection
        if way_chunks.len() < 3 { continue; }

        // remove dangeling ends
        // note: if a way is not directed, the dead ends could be visited to reach something interesting
        way_chunks.remove(0);
        way_chunks.pop();
    }
        
    
    
    todo!();    // Graph::new(nodes, edges);
}

/// Collect all [WayId]s of bikeable OpenStreetMap ways
fn bikeable_ways(ways: &HashMap<WayId, OsmWay>) -> Vec<WayId> { 
    let bikeable_ids: Vec<WayId> = ways
        .iter()
        .filter(|(_, way)| is_bikeable_way(&way))
        .map(|(id, _)| *id)
        .collect();
    
    bikeable_ids
}

/// Tries to determine if an OsmObj is routable in a blacklist fashion
/// note: Only Ways are routable, Nodes just give the way coordinates
fn is_bikeable_way(way: &OsmWay) -> bool {
    // whitelist legally allowed and passable highways
    // note: an OsmWay can also be an outline of a building
    let mut is_bikeable = false;
    for tag in way.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        // note: a trunk could have a bicycle lane
        // note: a primary road should only be choosen, if bicycle= is present
        // note: a footway only allowed if bicycle=yes (not checked atm)
        // note: a pedestrian only allowed if either bicycle=yes or vehicle=yes
        // note: a track with an undefined tracktype should be treated as worst case (tracktype=grade5)
        // note: a path without additional info could have a very bad surface
        if k == "highway" {
            match v {
                "primary" | "primary_link" | "secondary" | "secondary_link" | 
                "tertiary" | "tertiary_link" | "unclassified" | "residential" |
                "living_street" | "service" | "path" | "track" | "cycleway" | 
                "footway" | "pedestrian" => { is_bikeable = true; break; },
                _ => return false,
            }
        }
        
    }
    if ! is_bikeable { return false; }
    
    // check for additional conditions for making a way not bikable
    for tag in way.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        if k == "access" && v == "private" { return false; } // note: way could have bicycle=yes
        if k == "bicycle" {
            match v {
                "no" | "use_sidepath" => return false,
                _ => (),
            }
        }
        if k == "motorroad" && v == "yes" { return false; } // note: way could have cycleway=*
        if k == "tracktype" && v == "grade5" { return false; }
        if k == "smoothness" {
            match v {
                "very_bad" | "horrible" | "very_horrible" | "impassable" => return false,
                _ => (),
            }
        }
        if k == "surface" {
            match v {
                "stepping_stones" | "gravel" | "rock" |
                "pebblestone" | "mud" | "sand" | "woodclips" => return false,
                _ => (),
            }
        }
    }
    
    true
}

fn is_directed(way: &OsmWay) -> bool {
    
}

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

#[cfg(test)]
mod tests {
    use super::*;
            
    /// note: see https://github.com/chereskata/nice-bike-roundtrips-rs/blob/master/TAGS.md
    #[test]
    fn bikeable_ways_primary_combinations() {
        let mut data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );
        let bikeable_ways = bikeable_ways(&data.ways);
    
        // "highway=primary" with "bicycle=use_sidepath" is NOT bikeable
        // url: https://www.openstreetmap.org/way/4290108#map=18/51.49782/7.45615
        let id = 4290108;
        assert!(! bikeable_ways.contains(&id));
 
        // "highway=primary_link" with "bicycle=no" is NOT bikeable
        // url: https://www.openstreetmap.org/way/4071057#map=17/51.49929/7.46939
        let id = 4071057;
        assert!(! bikeable_ways.contains(&id));

        // "highway=primary_link" with "bicycle=use_sidepath" is NOT bikeable
        // url: https://www.openstreetmap.org/way/29030994#map=18/51.49687/7.44624
        let id = 29030994;
        assert!(! bikeable_ways.contains(&id));
    }

    /// note: see https://github.com/chereskata/nice-bike-roundtrips-rs/blob/master/TAGS.md
    #[test]
    fn bikeable_ways_track_combinations() {
        let mut data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );
        let bikeable_ways = bikeable_ways(&mut data.ways);
    
        // "highway=track" without any restrictions IS bikeable
        // url: https://www.openstreetmap.org/way/719650577#map=16/51.4879/7.4484
        let id = 719650577;
        assert!(bikeable_ways.contains(&id));
    }

    
    /// osmpbfreader::Way::nodes should be sorted from first node to last node
    /// in one direction, so every nodes neighbours are indeed listed before and
    /// after the node in Way::nodes
    #[test]
    fn _way_nodes_ordered() {
        let data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        // this way has additionally oneway=yes
        // url: https://www.openstreetmap.org/way/766725632#map=19/51.48087/7.44288
        let way_id: WayId = 766725632;
        // first node has i = 0 and last node has i = 14
        // the node ids are sorted from way's start to finish
        let real_node_ids: Vec<NodeId> = vec!(
            7160396711,
            9043910398,
            9043910397,
            9043910396,
            10203248997,
            9043910395,
            9043910394,
            3235121529,
            10203248995,
            9043910393,
            9730859168,
            3235121523,
            7128538365,
            7160502714,
            7160396771, 
        );

        let found_node_ids: Vec<NodeId> = data.ways.get(&way_id).unwrap().nodes
            .iter()
            .map(|node_id| node_id.0.unsigned_abs())
            .collect();
        
        // node count should be identical
        assert_eq!(real_node_ids.len(), found_node_ids.len());

        // the nodes shall be in the same (correct) order as node_ids
        for i in 0..real_node_ids.len() {
            assert_eq!(real_node_ids[i], found_node_ids[i]);            
        }
        
    }  
}
