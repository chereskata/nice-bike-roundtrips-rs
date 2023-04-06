use std::{fs::File, io::BufReader, process::exit, collections::HashMap, hash::Hash};

use osmpbfreader::{OsmPbfReader, OsmObj};

use crate::graph::Graph;

// For better readability of the code
/// Contains the OsmId of the OsmObj, simplified as a primitive
pub type OsmId = u64;

/// Build up a Graph from the routable part of OpenStreetMap data
pub fn weave(objs: &mut HashMap<OsmId, OsmObj>) -> Graph {    
    let mut ways: HashMap<OsmId, OsmObj> = bikeable_ways(objs);
    let mut nodes: HashMap<OsmId, OsmObj> = HashMap::new();

    // the value lets us cheaply tell, if the node is an intersection between
    // multiple ways, because every way stores it's node ids
    let mut node_ids: HashMap<OsmId, bool> = HashMap::new();

    // extract node ids of all bikeable ways into a single HashMap
    for (_, way) in ways.iter() {
        way.way().unwrap().nodes
            .iter()
            .for_each(|id| {
                let id = id.0 as u64;
                if node_ids.contains_key(&id) {
                    // node is an intersection
                    node_ids.insert(id, true);
                } else {
                    node_ids.insert(id, false);
                }
            });
    }

    // extract all nodes related to way sections into its own HashMap
    for id in node_ids.keys() {
        let entry = objs.remove_entry(&id).unwrap();
        nodes.insert(entry.0, entry.1);
    }
    
    let mut graph = Graph::new();
    
    // nodes have to be in the graph before the edges can be added
    todo!()
    
}

/// Removes all bike routing relevant [OsmObj]s from objs HashMap and extracts
/// them into their own one
fn bikeable_ways(objs: &mut HashMap<OsmId, OsmObj>) -> HashMap<OsmId, OsmObj> {    
    let mut bikeable_ways: HashMap<OsmId, OsmObj> = HashMap::new();

    let bikeable_ids: Vec<OsmId> = objs
        .iter()
        .filter(|(_, obj)| is_bikeable_way(&obj))
        .map(|(id, _)| id.clone())
        .collect();

    for id in bikeable_ids {
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
                "stepping_stones" | "gravel" | "rock" => return false,
                "pebblestone" | "mud" | "sand" | "woodclips" => return false,
                _ => (),
            }
        }
    }
    
    true
}

/// Returns a HashMap of every Node and Way in an pbf file.
/// note: Relations are ignored
pub fn map_from_pbf(path: &str) -> HashMap<OsmId, OsmObj> {
    let pbf = File::open(path)
            .expect("Could not find .pbf file");
    
    let mut buf = OsmPbfReader::new(BufReader::new(pbf));
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
    objs
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
    
    #[test]
    /// Check if bikeable_ways does not loose any OsmObjs while processing
    fn bikeable_ways_no_lost_objs() {
        let mut objs = map_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        let objs_count = objs.keys().count();

        // Gather results ...        
        let bikeable_ways = bikeable_ways(&mut objs);


        let non_bikeable_count = objs.keys().count();
        let bikeable_ways_count = bikeable_ways.keys().count();

        // Both 
        assert_eq!(objs_count, non_bikeable_count + bikeable_ways_count);
    }
            
    /// note: see https://github.com/chereskata/nice-bike-roundtrips-rs/blob/master/TAGS.md
    #[test]
    fn bikeable_ways_primary_combinations() {
        let mut objs = map_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );
        let bikeable_ways = bikeable_ways(&mut objs);
    
        // "highway=primary" with "bicycle=use_sidepath" is NOT bikeable
        // url: https://www.openstreetmap.org/way/4290108#map=18/51.49782/7.45615
        let id = 4290108;
        assert!(! bikeable_ways.contains_key(&id));
        assert!(objs.contains_key(&id));

        // "highway=primary_link" with "bicycle=no" is NOT bikeable
        // url: https://www.openstreetmap.org/way/4071057#map=17/51.49929/7.46939
        let id = 4071057;
        assert!(! bikeable_ways.contains_key(&id));
        assert!(objs.contains_key(&id));

        // "highway=primary_link" with "bicycle=use_sidepath" is NOT bikeable
        // url: https://www.openstreetmap.org/way/29030994#map=18/51.49687/7.44624
        let id = 29030994;
        assert!(! bikeable_ways.contains_key(&id));
        assert!(objs.contains_key(&id));
    }

    /// note: see https://github.com/chereskata/nice-bike-roundtrips-rs/blob/master/TAGS.md
    #[test]
    fn bikeable_ways_track_combinations() {
        let mut objs = map_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );
        let bikeable_ways = bikeable_ways(&mut objs);
    
        // "highway=track" without any restrictions IS bikeable
        // url: https://www.openstreetmap.org/way/719650577#map=16/51.4879/7.4484
        let id = 719650577;
        assert!(bikeable_ways.contains_key(&id));
        assert!(! objs.contains_key(&id));
    }

    // #[test] // Keep this test case because of the work of finding the info ...
    // fn nodes_of_way() {
    //     let mut objs = map_from_pbf(
    //         "resources/dortmund_sued.osm.pbf"
    //     );

    //     // url: https://www.openstreetmap.org/way/719650577#map=16/51.4879/7.4484
    //     let way_id = 719650577;
    //     let node_ids: Vec<u64> = vec!(
    //         6755537561,
    //         261724396,
    //         261724397,
    //         675132887,
    //         261724399,
    //         675133651,
    //         675133655,
    //         261724400,
    //         675133649,
    //         261724401,
    //         675133653,
    //         675133643,
    //         675133645,
    //         261724402
    //     );

    //     let nodes = super::nodes_of_way(&way_id, &mut objs);

    //     // node count should be identical
    //     assert_eq!(node_ids.len(), nodes.len());

    //     // note: identical lists have the same size and the same contents
    //     for id in node_ids {
    //         assert!(nodes.contains_key(&id));
    //     }
    // }
}
