use std::{fs::File, io::BufReader, process::exit, collections::HashMap};

use osmpbfreader::{OsmPbfReader, OsmObj, Node as OsmNode, Way as OsmWay};

use crate::graph::Graph;

mod data;

pub use crate::parser::data::*;

/// Build up a Graph from the routable part of OpenStreetMap data
pub fn weave(data: &mut OsmData) -> Graph {
    let mut ways: HashMap<WayId, OsmWay> = bikeable_ways(&mut data.ways);

    // nodes of the bikeable part of the street network
    // note: the value is an info, if the node is an intersection 
    let mut nodes: HashMap<NodeId, bool> = HashMap::new();

    // loop trough every way and read out his node ids
    // note: if a node is already registered in a previous iteration, multiple
    // ways cross it and it is an intersection
    for (_, way) in ways.iter() {
        way.nodes
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
    for (id, way) in ways.iter() {
        // collect all nodes from the way, shall be sorted
        let mut nodes_of_way: Vec<NodeId> = way.nodes
            .iter()
            .map(|node_id| node_id.0.unsigned_abs())
            .collect();

        // check, if the edge is directed
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

        // note: the `nodes` HashMaps could be freed of unneeded nodes and edges

        // the start could be dangling till the first intersection
        let mut i = 0;
        while i < nodes_of_way.len() {
            if *nodes.get(&nodes_of_way[i]).unwrap() {
                break;
            } else {
                i += 1;
            }
        }
        // remove the dangeling start
        for _ in 0..i {
            nodes_of_way.remove(0);
            // note: node could be removed from HashMaps here
        }

        // the end could be dangeling after the last intersection
        let mut i = nodes_of_way.len() - 1;
        while i >= 0 {
            if *nodes.get(&nodes_of_way[i]).unwrap() {
                break; 
            } else {
                i -= 1;
            }
        }
        // remove the dangeling end
        for _ in i..nodes_of_way.len() - 1 {
            nodes_of_way.pop();
            // note: node could be removed from HashMaps here
        }

        
        // if a way is unreachable, not one node is a intersection
        // if a way is a dead end, it has just one intersection node
        // 
        // => only one node left: the way has two dead ends and is useless as such
        // => no node left: the way is not reachable
        if nodes_of_way.len() < 2 { continue }

        // add the nodes to the to be Graph nodes HashMap
        nodes_of_way
            .iter()
            .for_each(|node_id| {
                if ! graph_edges.contains_key(&node_id) {
                    // graph_nodes.insert(
                    //     node_id
                    //     NODE,
                    //     tags,
                    //     edges,
                    //     greatness
                    // ));
                }
            });

        // split the way in parts between intersections with other ways        
    }

    
    
    todo!();    // Graph::new(nodes, edges);
}

/// Removes all bike routing relevant [OsmWay]s from ways HashMap and extracts
/// them into their own one
fn bikeable_ways(ways: &mut HashMap<WayId, OsmWay>) -> HashMap<WayId, OsmWay> { 
    let mut bikeable_ways: HashMap<WayId, OsmWay> = HashMap::new();

    let bikeable_ids: Vec<WayId> = ways
        .iter()
        .filter(|(_, way)| is_bikeable_way(&way))
        .map(|(id, _)| *id)
        .collect();

    for id in bikeable_ids {
        let entry = ways.remove_entry(&id).unwrap();
        bikeable_ways.insert(entry.0, entry.1);
    }
    
    bikeable_ways
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

/// 
// fn nodes_of_way() -> Vec<OsmId> {
    
// }

/// Returns a container of every Node, Way and Realation in an pbf file.
pub fn data_from_pbf(path: &str) -> OsmData {
    let pbf = File::open(path)
            .expect("Could not find .pbf file");
    
    let mut buf = OsmPbfReader::new(BufReader::new(pbf));

    let mut data = OsmData::new();
    for chunk in buf.iter() {
        match chunk {
            Ok(obj) => 
                match obj {
                    OsmObj::Node(n) => {
                        data.nodes.insert(n.id.0.unsigned_abs(), n);
                    },
                    OsmObj::Way(w) => {
                        data.ways.insert(w.id.0.unsigned_abs(), w);
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
    
    #[test]
    /// Check if bikeable_ways does not loose any OsmObjs while processing
    fn bikeable_ways_no_lost_objs() {
        let mut data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        let ways_count = data.ways.keys().count();

        // Gather results ...        
        let bikeable_ways = bikeable_ways(&mut data.ways);


        let non_bikeable_count = data.ways.keys().count();
        let bikeable_ways_count = bikeable_ways.keys().count();

        // Both 
        assert_eq!(ways_count, non_bikeable_count + bikeable_ways_count);
    }
            
    /// note: see https://github.com/chereskata/nice-bike-roundtrips-rs/blob/master/TAGS.md
    #[test]
    fn bikeable_ways_primary_combinations() {
        let mut data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );
        let bikeable_ways = bikeable_ways(&mut data.ways);
    
        // "highway=primary" with "bicycle=use_sidepath" is NOT bikeable
        // url: https://www.openstreetmap.org/way/4290108#map=18/51.49782/7.45615
        let id = 4290108;
        assert!(! bikeable_ways.contains_key(&id));
        assert!(data.ways.contains_key(&id));

        // "highway=primary_link" with "bicycle=no" is NOT bikeable
        // url: https://www.openstreetmap.org/way/4071057#map=17/51.49929/7.46939
        let id = 4071057;
        assert!(! bikeable_ways.contains_key(&id));
        assert!(data.ways.contains_key(&id));

        // "highway=primary_link" with "bicycle=use_sidepath" is NOT bikeable
        // url: https://www.openstreetmap.org/way/29030994#map=18/51.49687/7.44624
        let id = 29030994;
        assert!(! bikeable_ways.contains_key(&id));
        assert!(data.ways.contains_key(&id));
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
        assert!(bikeable_ways.contains_key(&id));
        assert!(! data.ways.contains_key(&id));
    }

    
    // / osmpbfreader::Way::nodes should be sorted from first node to last node
    // / in one direction, so every nodes neighbours are indeed listed before and
    // / after the node in Way::nodes
    // #[test]
    // fn _way_nodes_ordered() {
    //     let mut objs = map_from_pbf(
    //         "resources/dortmund_sued.osm.pbf"
    //     );

    //     // url: https://www.openstreetmap.org/way/766725632#map=19/51.48087/7.44288
    //     let way_id = 766725632;
    //     // first node has i = 0 and last node has i = 14
    //     // the node ids are sorted from way's start to finish
    //     let node_ids: Vec<u64> = vec!(
    //         7160396711,
    //         9043910398,
    //         9043910397,
    //         9043910396,
    //         10203248997,
    //         9043910395,
    //         9043910394,
    //         3235121529,
    //         10203248995,
    //         9043910393,
    //         9730859168,
    //         3235121523,
    //         7128538365,
    //         7160502714,
    //         7160396771, 
    //     );

        

        
    //     // node count should be identical
    //     assert_eq!(node_ids.len(), nodes.len());

    //     // the nodes shall be in the same (correct) order as node_ids
    //     for i in 0..node_ids.len() {
        
    //     }
        
    // }    
    
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
