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

// returns empty vector if no path exists between the two nodes
fn a_star(graph: &Graph, start: &NodeId, end: &NodeId) -> Option<Vec<NodeId>> {
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
        println!("current node is {}", node_id);
        if node_id == *end {
            // collect path from start to end
            let mut path: Vec<NodeId> = Vec::new();
            let mut current: NodeId = current.0;
            path.push(current);
            while came_from.contains_key(&current) {
                current = *came_from.get(&current).unwrap();
                path.push(current);
            }
            path.reverse();
            return Some(path);
        }
        let node = graph.nodes().get(&node_id).unwrap();

        println!("node has {} edges", node.edges().len());
        
        for edge_id in node.edges() {
            println!("edge id {}", edge_id);
            // find other end of edge
            let edge = graph.edges().get(&edge_id).unwrap();
            let neighbour_node_id: NodeId;
            if *edge.s() == node_id {
                neighbour_node_id = *edge.t();
            } else if *edge.directed() {
                // this edge is directed, and [node] is at its end (node_id == t)
                // neigbour_node is not directly reachable from [node]
                continue;
            } else {
                neighbour_node_id = *edge.s();
            }
            
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
    None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{*, Node as GraphNode, Edge as GraphEdge};

    #[test]
    fn a_star_no_path() {
        // start: {1, 2}
        // end: 0
        //
        // 0--->1<-->2

        const DISTANCE: f64 = 1.0;
        let mut graph_nodes: HashMap<NodeId, GraphNode> = HashMap::new();
        graph_nodes.insert(0, GraphNode::new(0, Point::new(0.0, 0.0)));
        graph_nodes.insert(1, GraphNode::new(1, Point::new(1.0, 0.0)));
        graph_nodes.insert(2, GraphNode::new(2, Point::new(2.0, 0.0)));

        let mut graph_edges: HashMap<EdgeId, GraphEdge> = HashMap::new();
        graph_edges.insert(0, GraphEdge::new(0, DISTANCE, true, vec![0, 1]));
        graph_nodes.get_mut(&0).unwrap().insert_edge(0);
        graph_nodes.get_mut(&1).unwrap().insert_edge(0);
        graph_edges.insert(1, GraphEdge::new(1, DISTANCE, false, vec![1, 2]));
        graph_nodes.get_mut(&1).unwrap().insert_edge(1);
        graph_nodes.get_mut(&2).unwrap().insert_edge(1);

        let graph = Graph::new(graph_nodes, graph_edges);

        let result = a_star(&graph, &0, &1);
        assert_eq!(None, result);

        let result = a_star(&graph, &2, &0);
        assert_eq!(None, result);
    }

    #[test]
    fn a_star_simple_path() {
        // start: 0
        // end: 6
        //
        // 0--->1<-->2--->3<-->8
        // |    |         |
        // |    |         |
        // 4<---5<---6<-->7

        let mut graph_nodes: HashMap<NodeId, GraphNode> = HashMap::new();
        graph_nodes.insert(0, GraphNode::new(0, Point::new(0.0, 1.0)));
        graph_nodes.insert(1, GraphNode::new(1, Point::new(1.0, 1.0)));
        graph_nodes.insert(2, GraphNode::new(2, Point::new(2.0, 1.0)));
        graph_nodes.insert(3, GraphNode::new(3, Point::new(3.0, 1.0)));
        graph_nodes.insert(8, GraphNode::new(8, Point::new(4.0, 1.0)));
        graph_nodes.insert(4, GraphNode::new(4, Point::new(0.0, 0.0)));
        graph_nodes.insert(5, GraphNode::new(5, Point::new(0.0, 1.0)));
        graph_nodes.insert(6, GraphNode::new(6, Point::new(0.0, 2.0)));
        graph_nodes.insert(7, GraphNode::new(7, Point::new(0.0, 3.0)));

        let mut graph_edges: HashMap<EdgeId, GraphEdge> = HashMap::new();
        graph_edges.insert(0, GraphEdge::new(0, 1.0, true, vec![0, 1]));
        graph_nodes.get_mut(&0).unwrap().insert_edge(0);
        graph_nodes.get_mut(&1).unwrap().insert_edge(0);
        graph_edges.insert(1, GraphEdge::new(1, 1.0, false, vec![1, 2]));
        graph_nodes.get_mut(&1).unwrap().insert_edge(1);
        graph_nodes.get_mut(&2).unwrap().insert_edge(1);
        graph_edges.insert(2, GraphEdge::new(2, 1.0, true, vec![2, 3]));
        graph_nodes.get_mut(&2).unwrap().insert_edge(2);
        graph_nodes.get_mut(&3).unwrap().insert_edge(2);
        graph_edges.insert(3, GraphEdge::new(3, 1.0, false, vec![3, 8]));
        graph_nodes.get_mut(&3).unwrap().insert_edge(3);
        graph_nodes.get_mut(&8).unwrap().insert_edge(3);
        graph_edges.insert(4, GraphEdge::new(4, 1.0, false, vec![0, 4]));
        graph_nodes.get_mut(&0).unwrap().insert_edge(4);
        graph_nodes.get_mut(&4).unwrap().insert_edge(4);
        graph_edges.insert(5, GraphEdge::new(5, 1.0, false, vec![1, 5]));
        graph_nodes.get_mut(&1).unwrap().insert_edge(1);
        graph_nodes.get_mut(&5).unwrap().insert_edge(5);
        graph_edges.insert(6, GraphEdge::new(6, 1.0, false, vec![7, 3])); // logical s, t are reversed
        graph_nodes.get_mut(&7).unwrap().insert_edge(6);
        graph_nodes.get_mut(&3).unwrap().insert_edge(6);
        graph_edges.insert(7, GraphEdge::new(7, 1.0, true, vec![5, 4]));
        graph_nodes.get_mut(&5).unwrap().insert_edge(7);
        graph_nodes.get_mut(&4).unwrap().insert_edge(7);
        graph_edges.insert(8, GraphEdge::new(8, 1.0, true, vec![6, 5]));
        graph_nodes.get_mut(&6).unwrap().insert_edge(8);
        graph_nodes.get_mut(&5).unwrap().insert_edge(8);
        graph_edges.insert(9, GraphEdge::new(9, 1.0, false, vec![6, 7]));
        graph_nodes.get_mut(&6).unwrap().insert_edge(9);
        graph_nodes.get_mut(&7).unwrap().insert_edge(9);

        let graph = Graph::new(graph_nodes, graph_edges);
        
        let result = a_star(&graph, &0, &6).unwrap();
        let should_be = vec![0, 1, 2, 3, 7, 6];

        assert_eq!(should_be, result);
    }
}