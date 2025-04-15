use std::collections::HashSet;
use std::str::FromStr;

use geo::Point;
use gpx::Gpx;
use gpx::Route;
use gpx::Waypoint;

use crate::graph::EdgeId;
use crate::graph::Graph;
use crate::graph::NodeId;
use crate::graph::Edge as GraphEdge;

pub fn intersections_to_gpx(graph: &Graph, main_nodes: &Vec<NodeId>) -> Gpx {
    let mut waypoints: Vec<Waypoint> = Vec::new();

    let first = graph.nodes().get(&main_nodes[0]).unwrap();
    waypoints.push(Waypoint::new(*first.point()));
    for i in 1..main_nodes.len() {
        let points = intermediary_points(graph, &main_nodes[i - 1], &main_nodes[i]);
        waypoints.append(&mut points
            .iter()
            .map(|p| Waypoint::new(*p))
            .collect()
        );

        let i_th = graph.nodes().get(&main_nodes[i]).unwrap();
        waypoints.push(Waypoint::new(*i_th.point()));
    }

    let route = Route {
        name: Some(String::from_str("roundtrip").unwrap()),
        comment: None,
        description: None,
        source: None,
        links: Vec::new(),
        number: None,
        type_: None,
        points: waypoints
    };
    
    Gpx {
        version: gpx::GpxVersion::Gpx11,
        creator: Some(String::from_str("nice-bike-roundtrips-rs").unwrap()),
        metadata: None,
        waypoints: Vec::new(),
        tracks: Vec::new(),
        routes: vec![route],
    }
}


// all points that are between from and to, sorted in correct diretion
fn intermediary_points(graph: &Graph, from: &NodeId, to: &NodeId) -> Vec<Point> {
    let from_edges: HashSet<EdgeId> = edges_to_hs(graph, from);
    let to_edges: HashSet<EdgeId> = edges_to_hs(graph, to);

    let shared_edge = from_edges
        .intersection(&to_edges)
        .into_iter()
        .fold(None, |result: Option<EdgeId>, edge_id| {
            match result {
                None => Some(*edge_id),
                Some(_) => result,
                // Some(_) => panic!(), // there exist more than one edge ...
            }
        })
        .unwrap();

    let shared_edge = graph.edges().get(&shared_edge).unwrap();

    let from_index = position_in_edge(&shared_edge, &from);
    let to_index = position_in_edge(&shared_edge, to);

    let intermediary: Vec<Point>;
    if from_index > to_index {
        intermediary = shared_edge
            .nodes()
            .iter()
            .enumerate()
            .filter_map(|(i, node_id)| {
                // we the way in reverse, so the index names are a bit unintuitive
                if i > to_index && i < from_index {
                    return Some(graph.nodes().get(node_id).unwrap().point().clone());
                }
                None
            })
            .rev()
            .collect()
    } else {
        intermediary = shared_edge
            .nodes()
            .iter()
            .enumerate()
            .filter_map(|(i, node_id)| {
                if i > from_index && i < to_index {
                    return Some(graph.nodes().get(node_id).unwrap().point().clone());
                }
                None
            })
            .collect();
    }

    intermediary
}

/// the edges of a node as HashSet
fn edges_to_hs(graph: &Graph, node: &NodeId) -> HashSet<EdgeId> {
    graph
        .nodes()
        .get(node)
        .unwrap()
        .edges()
        .iter()
        .copied()
        .collect()
}

// index of the node in the edge
fn position_in_edge(edge: &GraphEdge, target: &NodeId) -> usize {
    edge
        .nodes()
        .iter()
        .enumerate()
        .fold(None, |at: Option<usize>, (index, node_id)| {
            match at {
                Some(_) => at,
                None => {
                    if *node_id == *target { return Some(index); }
                    at
                },
            }
        })
        .unwrap()
}