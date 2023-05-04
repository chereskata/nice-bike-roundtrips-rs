use std::collections::HashMap;

use geo::Point;
use geo::EuclideanDistance;

use crate::graph::NodeId;
use crate::{graph::Graph, Config};

pub fn unoptimized(graph: &Graph, visit: &Vec<NodeId>, start: &NodeId) -> Vec<NodeId> {
    
    
    todo!()
}


fn dijkstra(graph: &Graph, start: &NodeId, end: &NodeId) -> Vec<NodeId> {
    pub type Distance = f64;  
    
    let mut routing_table: HashMap<NodeId, Distance> = HashMap::new();
    routing_table.insert(*start, 0_f64);

    let mut route: Vec<NodeId> = vec![*start];
    while ! routing_table.keys().any(|dest| *dest == *end) {
        
    }

    todo!();
}

fn a_star(graph: &Graph, start: &NodeId, end: &NodeId) -> Vec<NodeId> {

    todo!();
}

/// Closest point by euclidean distance
pub fn closest_point(graph: &Graph, p: &Point) -> NodeId {
    let distance: f64 = f64::MAX;
    let mut nearest = None;
    for node in graph.nodes() {
        if distance > p.euclidean_distance(node.1.point()) {
            nearest = Some(*node.0);
        }
    }
    nearest.unwrap()
}

pub fn nearest_graph_nodes(graph: &Graph, points: &Vec<Point>) -> Vec<NodeId> {
    let ids: Vec<NodeId> = points
        .iter()
        .map(|p| closest_point(&graph, &p))
        .collect();
    ids
}