use geo::Point;
use geo::EuclideanDistance;

use crate::graph::NodeId;
use crate::{graph::Graph, Config};

pub fn unoptimized(graph: &Graph, config: &Config) -> () {
    let start = Point::new(
        config.start_lat.clone(),
        config.start_lon.clone()
    );

    let start: NodeId = closest_point(&graph, &start);

    
    
    todo!();
}

/// Closest point by euclidean distance
fn closest_point(graph: &Graph, p: &Point) -> NodeId {
    let distance: f64 = f64::MAX;
    let mut nearest = None;
    for node in graph.nodes() {
        if distance > p.euclidean_distance(node.1.point()) {
            nearest = Some(*node.0);
        }
    }
    nearest.unwrap()
}