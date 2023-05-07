use std::cmp::Reverse;
use std::collections::HashMap;

use geo::Point;
use geo::EuclideanDistance;
use ordered_float::NotNan;
use priority_queue::PriorityQueue;

use crate::graph::{Graph, NodeId};

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
    // key == node, value == predecessor
    let mut came_from: HashMap<NodeId, NodeId> = HashMap::new();

    // least known distance from start to key
    let mut g_score: HashMap<NodeId, f64> = HashMap::new();
    g_score.insert(*start, 0_f64);

    // heuristic of distance from start via=key to end
    let h = Point::euclidean_distance(
        graph.nodes().get(start).unwrap().point(),
        graph.nodes().get(end).unwrap().point()
    );
    let mut f_score: HashMap<NodeId, f64> = HashMap::new();
    f_score.insert(*start, h);

    let mut open_set: PriorityQueue<NodeId, Reverse<NotNan<f64>>> = PriorityQueue::new();
    open_set.push(*start, Reverse(NotNan::new(h).unwrap()));

    while ! open_set.is_empty() {
        let current = open_set.pop().unwrap();
        let node_id = current.0;
        if node_id == *end {
            let mut path: Vec<NodeId> = Vec::new();
            let mut current: NodeId = current.0;
            path.push(current);
            while came_from.contains_key(&current) {
                current = *came_from.get(&current).unwrap();
                path.push(current);
            }
            path.reverse();
            return path;
        }
        let node = graph.nodes().get(&node_id).unwrap();
        
        for edge_id in node.edges() {
            // find other end of edge
            let edge = graph.edges().get(&edge_id).unwrap();
            let mut neighbour_node_id = None;
            if *edge.s() == node_id {
                neighbour_node_id = Some(*edge.t());
            } else {
                neighbour_node_id = Some(*edge.s());
            }
            let neighbour_node_id = neighbour_node_id.unwrap();
            
            let tentative_g_score = g_score.get(&node_id).unwrap() + edge.distance();

            if tentative_g_score < *g_score.get(&neighbour_node_id).unwrap_or(&f64::MAX) {
                came_from.insert(neighbour_node_id, node_id);
                g_score.insert(neighbour_node_id, tentative_g_score);
                
                let h = Point::euclidean_distance(
                    graph.nodes().get(&neighbour_node_id).unwrap().point(),
                    graph.nodes().get(end).unwrap().point()
                );
                
                let f = tentative_g_score + h; 
                f_score.insert(neighbour_node_id, f);

                open_set.change_priority(&neighbour_node_id, Reverse(NotNan::new(f).unwrap()));
            }
        }
        
    }

    panic!("Unreachable code");
}

/// Closest point by euclidean distance
pub fn closest_point(graph: &Graph, p: &Point) -> NodeId {
    // maybe use harvesine distance instead of euclidean
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