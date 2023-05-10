use osmpbfreader::{Node as OsmNode, Way as OsmWay};
use std::collections::HashMap;

use crate::graph::{Graph, NodeId, EdgeId, Node as GraphNode, Edge as GraphEdge};
use crate::parser::data::*;

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
    let mut graph_nodes: HashMap<NodeId, GraphNode> = HashMap::new();
    let mut graph_edges: HashMap<EdgeId, GraphEdge> = HashMap::new();

    for way_id in ways.iter() {
        let way = data.ways.get(&way_id).unwrap();
        
        // split way at every intersection
        let way_chunks: Vec<Vec<NodeId>>;
        match chunk_up(&nodes, &way) {
            Some(wc) => way_chunks = wc,
            None => continue,
        }
        
        use geo::Point;
        
        // use node chunks to create an GraphEdge and register its Nodes
        way_chunks.iter().enumerate().for_each(|(i, chunk)| {
            // note: shifting could be coded in global variable instead
            let edge_id = ((i as u64) << 53u64) | way_id;

            let mut distance: f64 = 0.0;
            let mut points: (Option<Point>, Option<Point>) = (None, None);
            for j in 0..chunk.len() { // chunk.len() not included in enumeration
                let node_id = chunk[j];
                let node = data.nodes.get(&node_id).unwrap();
                let point: Point = point_from(&node);

                // coordinates of previous point and current point
                points = (points.1, Some(point));

                // measure distance
                match points {
                    (Some(p1), Some(p2)) => {
                        distance += geo::VincentyDistance::vincenty_distance(&p1, &p2).unwrap_or(0.0);
                    },
                    _ => (),
                }
                
                // create new GraphNode if neccessary
                if ! graph_nodes.contains_key(&node_id) {
                    graph_nodes.insert(
                        node_id,
                        GraphNode::new(
                            node_id,
                            point.clone()
                        )
                    );
                }
               
                // register this way chunk in GraphNode
                graph_nodes.get_mut(&node_id).unwrap().insert_edge(edge_id);
            }
            
            // Create Edge from WayChunk
            graph_edges.insert(
                edge_id,
                GraphEdge::new(
                    edge_id,
                    distance,
                    is_directed(&way),
                    chunk.clone() // note: cloning is not very nice
                )
            );
        });
    }

    Graph::new(graph_nodes, graph_edges)
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

/// If some way has more than one intersection, its non dead end chunks will be returned
/// note: a way could still lead to nowhere, when any intersection finally leads to nowhere
fn chunk_up(nodes: &HashMap<NodeId, bool>, way: &OsmWay) -> Option<Vec<Vec<NodeId>>> {
    // discover way's nodes, shall be sorted from start to end
    let nodes_of_way: Vec<NodeId> = way.nodes
        .iter()
        .map(|node_id| node_id.0.unsigned_abs())
        .collect();

    let mut way_chunks: Vec<Vec<NodeId>> = Vec::new();

    let mut chunk: Vec<NodeId> = Vec::new();
    for node_id in nodes_of_way {
        chunk.push(node_id);
        // if node is an intersection
        if *nodes.get(&node_id).unwrap() {
            // split the way here
            way_chunks.push(chunk); // left split contains intersection
            chunk = Vec::new();
            chunk.push(node_id); // right split contains intersection
        }
    };
    way_chunks.push(chunk); // last chunk can never be pushed inside the loop
    

    // remove dangeling ends
    // note: if a way is not directed, the dead ends could be visited to reach something interesting
    way_chunks.remove(0); // till the first intersection
    way_chunks.pop(); // from the last intersection to the end
    
    if way_chunks.len() < 1 { return None; } // either not connected or both directions are dead ends
    Some(way_chunks)
}

fn is_directed(way: &OsmWay) -> bool {
    way.tags
        .iter()
        .any(|tag| {
            if tag.0.as_str() == "oneway" {
                match tag.1.as_str() {
                    "yes" | "1" | "true" => return true,
                    _ => return false
                }
            }
            false
    })
}

fn point_from(node: &OsmNode) -> geo::Point {
    geo::Point::new(node.lat(), node.lon())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::data_from_pbf;
    
    /// note: see https://github.com/chereskata/nice-bike-roundtrips-rs/blob/master/TAGS.md
    #[test]
    fn bikeable_ways_primary_combinations() {
        let data = data_from_pbf(
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
    
    #[test]
    fn graph_node_knows_ways() {
        let data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        let node_id: NodeId = 271456407;
        let ways_should_be: Vec<WayId> = vec![       
            910466050,
            37179867
        ];
        let edges_should_be: Vec<EdgeId> = vec![
            9007200165207042,
            910466050,
            54043195565625819
        ];
        
        let graph = super::weave(&data);
        let node = graph.nodes().get(&node_id).unwrap();

        let edges_result: Vec<EdgeId> = node.edges().clone();
        let mut ways_result: Vec<WayId> = edges_result.clone()
            .iter()
            .map(|edge_id| (*edge_id << 11u64) >> 11u64)
            .collect();
        // split ways still have the same way id
        ways_result.dedup();
        
        assert_eq!(ways_should_be.len(), ways_result.len());
        for way_id in ways_should_be {
            assert!(ways_result.contains(&way_id));
        }

        assert_eq!(edges_should_be.len(), edges_result.len());
        for edge_id in edges_should_be {
            assert!(edges_result.contains(&edge_id));
        }

    }

    /// currently the graph does not contain some nodes and ways
    #[test]
    fn graph_no_lost_ways() {
        let data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );
    
        let graph = super::weave(&data);
        assert_eq!(true, graph.edges().keys().into_iter().any(|edge_id| ((*edge_id << 11u64) >> 11u64) == 25750400));
        assert_eq!(true, graph.edges().keys().into_iter().any(|edge_id| ((*edge_id << 11u64) >> 11u64) == 25750400));
        assert_eq!(true, graph.edges().keys().into_iter().any(|edge_id| ((*edge_id << 11u64) >> 11u64) == 203970758));
        assert_eq!(true, graph.edges().keys().into_iter().any(|edge_id| ((*edge_id << 11u64) >> 11u64) == 910466050));
        assert_eq!(true, graph.edges().keys().into_iter().any(|edge_id| ((*edge_id << 11u64) >> 11u64) == 37179867));        
        assert_eq!(true, graph.nodes().keys().into_iter().any(|node_id| *node_id == 280824608));

        // this way is a dead end, because one end of it are steps, so unpassable by bike
        assert_eq!(false, graph.edges().keys().into_iter().any(|edge_id| ((*edge_id << 11u64) >> 11u64) == 52060549))
    }
    
    /// Build a graph from all highways inside "Naturschutzgebiet Bolmke",
    /// OpenStreetMap id: 964278663
    #[test]
    fn weave_bolmke() {
        // Ways were gathered using Overpass Turbo query:
        // // [out:json];
        // // area[name="NSG Bolmke"];
        // // way["highway"](area);
        // // (._;>;);
        // // out;

        // value designates, if a way should be inside the graph
        // additionally add way #517081281,
        // url: https://www.openstreetmap.org/way/517081281 to have a more
        // complex topology
        let mut all_bolmke_ways: HashMap<WayId,bool> = HashMap::from([
            (517081281, true),
            //
            (24163197, true),
            (24163200, true),
            (24205888, true),
            (24212287, true),
            (24212293, true),
            (24212306, true),
            (24212332, true),
            (24212416, true),
            (24212422, true),
            (24212425, true),
            (24212440, true),
            (25112902, true),
            (25334372, true),
            (25750397, true),
            (25750399, true),
            (25839122, true),
            (26442301, true),
            (26762746, true),
            (26762747, true),
            (34491573, true),
            (37179828, true),
            (37179867, true),
            (37179869, true),
            (37179870, true),
            (38084645, true),
            (38084646, true),
            (38084647, true),
            (38084648, true),
            (39914724, true),
            (39947912, true),
            (53396595, true),
            (53396597, true),
            (53396598, true),
            (53560358, true),
            (326987556, true),
            (487599223, true),
            (492512071, true),
            (492512072, true),
            (492512073, true),
            (492512074, true),
            (719650577, true),
            (889518538, true),
            (910466050, true),
            (931345543, true),
            (1025976827, true),
            (1029482849, true),
            (1036677995, true),
            (1044807538, true),
        ]);
        
        // additionally the graph should not contain dead ends, so filter non
        // circular ways out, to reflect the graph contents
        
        // leads to way outside of Bolmke NSG
        // url: https://www.openstreetmap.org/way/39914724#map=19/51.49048/7.44303
        all_bolmke_ways.entry(39914724).and_modify(|in_graph| *in_graph = false);
        // dead end
        // url: https://www.openstreetmap.org/way/1025976827
        all_bolmke_ways.entry(1025976827).and_modify(|in_graph| *in_graph = false);
        

        // note: only leaf dead ends are filtered from the map. Higher level dead ends are not
        // leads to way outside of Bolmke NSG
        // url: https://www.openstreetmap.org/way/24212287#map=18/51.49012/7.44321
        // all_bolmke_ways.entry(24212287).and_modify(|in_graph| *in_graph = false);
        
        // leads to way outside of Bolmke NSG
        // url: https://www.openstreetmap.org/way/53560358
        all_bolmke_ways.entry(53560358).and_modify(|in_graph| *in_graph = false);
        // leads to way outside of Bolmke NSG
        // url: https://www.openstreetmap.org/way/326987556#map=19/51.48040/7.45259
        all_bolmke_ways.entry(326987556).and_modify(|in_graph| *in_graph = false);
        // leads to way outside of Bolmke NSG
        // url: https://www.openstreetmap.org/way/37179870
        all_bolmke_ways.entry(37179870).and_modify(|in_graph| *in_graph = false);
        // leads to outside of Bolmke NSG
        // url: https://www.openstreetmap.org/way/889518538
        all_bolmke_ways.entry(889518538).and_modify(|in_graph| *in_graph = false);



        // These nodes sould have been cut off, because they belong to dead end ways
        let mut illegal_nodes: Vec<NodeId> = Vec::new();

        // note: way #24212416 should have its leftmost nodes removed
        // url: https://www.openstreetmap.org/way/24212416#map=19/51.48468/7.44724
        illegal_nodes.push(673992475);
        illegal_nodes.push(673992445);

        // note: way #24212440 should have its downmost nodes removed
        // url: https://www.openstreetmap.org/way/24212440#map=18/51.47985/7.44770
        // non legal nodes have ids: 
        illegal_nodes.push(9669606440);
        illegal_nodes.push(676631001);
        illegal_nodes.push(262165276);
        illegal_nodes.push(262165277);
        illegal_nodes.push(262165278);
        illegal_nodes.push(262165279);
        illegal_nodes.push(676630974);
        illegal_nodes.push(262165280);
        illegal_nodes.push(676630938);
        illegal_nodes.push(262165281);
        illegal_nodes.push(262165282);

        // note: way #37179867
        // url: https://www.openstreetmap.org/way/37179867#map=19/51.47736/7.45304
        illegal_nodes.push(675213222);
        illegal_nodes.push(675213208);
        
        // note: way #25750399
        // url: https://www.openstreetmap.org/way/25750399
        // non legal nodes have ids:
        illegal_nodes.push(280824768);
        illegal_nodes.push(280824774);
        illegal_nodes.push(288417434);
        
        // note: way #910466050
        // url: https://www.openstreetmap.org/way/910466050
        // non legal nodes have ids:
        illegal_nodes.push(262163628);
        illegal_nodes.push(262163630);
        illegal_nodes.push(280824946);


        // note: way #24212293
        // url: https://www.openstreetmap.org/way/24212293
        // non legal nodes have ids:
        illegal_nodes.push(675142881);

        // note: way #39947912
        // url: https://www.openstreetmap.org/way/39947912#map=18/51.48856/7.44968
        // non legal nodes have ids:
        illegal_nodes.push(262163418); // only first level leaves (no recursiveness implemented when searching leaves)
        // illegal_nodes.push(8269731798);
        // illegal_nodes.push(479732695);
        // illegal_nodes.push(674954760);
        // illegal_nodes.push(479732696);
        // illegal_nodes.push(674954748);
        // illegal_nodes.push(479732697);
        // illegal_nodes.push(674954765);
        // illegal_nodes.push(479732698);
        // illegal_nodes.push(702312703);
        // illegal_nodes.push(674956272);
        


        // BEGIN TESTING
        let mut data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        // remove all ways, except the ones listed above
        data.ways.retain(|way_id, _| all_bolmke_ways.contains_key(way_id));

        let graph = super::weave(&data);

        // each graph way should originate from one OpenStreetMap way, that could be split
        for (edge_id, _) in graph.edges() {
            // edge should be in Bolmke NSG
            let way_id = (edge_id << 11u64) >> 11u64; // highest 11 bits are the chunk id
            assert!(all_bolmke_ways.contains_key(&way_id));
            assert_eq!(all_bolmke_ways.get(&way_id), Some(&true));
        }

        // each node should not be blacklisted
        for (node_id, _) in graph.nodes() {
            assert_eq!(illegal_nodes.contains(node_id), false);
        }
    }
}
