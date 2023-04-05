use std::{fs::File, io::BufReader, process::exit, collections::HashMap};

use osmpbfreader::{OsmPbfReader, OsmObj};

use crate::graph::Graph;

// For better readability of the code
/// Contains the OsmId of the OsmObj, simplified as a primitive
pub type OsmId = u64;

/// Build up a Graph from the OpenStreetMap data
pub fn weave(pbf: File) -> Graph {
    let mut buf = OsmPbfReader::new(BufReader::new(pbf));
    
    // Parse all OpenStreetMap nodes and ways. Ignore relations
    // note: Relations can yield useful meta knowledge about intersting paths
    let mut objs: HashMap<OsmId, OsmObj> = buf
        .iter()
        .filter_map(|r| {
            // Only real OsmObjects shall remain
            match r {
                Ok(obj) => Some(obj),
                Err(e) => {
                    eprintln!("{:?}", e);
                    None
                },
            }
        })
        .filter(|obj| ! matches!(obj, OsmObj::Relation(_)) )
        .map(|obj| (obj.id().inner_id() as u64, obj) )
        .collect();

    let mut bikeable_ways: HashMap<OsmId, OsmObj> = bikeable_ways(&mut objs);

    for (_, v) in bikeable_ways {
        print_object(&v);
    }
    
    let mut graph = Graph::new();

    // nodes have to be in the graph before the edges can be added
    todo!()
    
}

/// Removes all bike routing relevant [OsmObj]s from objs Vector and extracts
/// them into their own one
fn bikeable_ways(objs: &mut HashMap<OsmId, OsmObj>) -> HashMap<OsmId, OsmObj> {    
    let mut bikeable_ways: HashMap<OsmId, OsmObj> = HashMap::new();
    
    let bikeable_keys: Vec<OsmId> = objs
        .iter()
        .filter(|(_, obj)| is_bikeable_way(&obj))
        .map(|(id, _)| id.clone())
        .collect();

    for id in bikeable_keys {
        let entry = objs.remove_entry(&id).unwrap();
        bikeable_ways.insert(entry.0, entry.1);
    }
    
    bikeable_ways
}

/// Tries to determine if an OsmObj is routable in a blacklist fashion
/// note: Only Ways are routable, Nodes just give the way coordinates
fn is_bikeable_way(obj: &OsmObj) -> bool {
    let way = match obj {
        OsmObj::Node(_) | OsmObj::Relation(_) => return false,
        OsmObj::Way(way) => way,
    };

    // note: Traversing the tags two times does not seem optimal for me
    // check if the way is a passable highway
    // if ! way.tags.iter().any(|tag| tag.0.as_str() == "highway") { return false; }

    // whitelist legally allowed and passable highways
    let mut is_bikeable = false;
    for tag in way.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        // note: trunks and trunk_links could have cycleway by its side
        if k == "highway" {
            match v {
                "primary" | "primary_link" | "secondary" | "secondary_link" | 
                "tertiary" | "tertiary_link" | "unclassified" | "residential" |
                "living_street" | "service" | "path" | "track" | "bridleway" |
                "cycleway" | "footway" | "pedestrian" => { is_bikeable = true; break; },
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
        if k == "bicycle" && v == "no" { return false; }
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
                "stepping_stones" | "gravel" | "rock" => return false,
                "pebblestone" | "mud" | "sand" | "woodclips" => return false,
                _ => (),
            }
        }
    }
    
    true
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