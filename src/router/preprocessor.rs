use std::cmp::Reverse;

use geo::ConcaveHull;
use geo::LineString;
use geo::Point;
use geo::EuclideanDistance;
use ordered_float::NotNan;
use priority_queue::PriorityQueue;

use crate::graph::{Graph, NodeId};

/// orders interesting nodes based on their location in a concave hull
/// note: the inner nodes of a the hull are not used yet
pub fn order_with_concave_hull(graph: &Graph, start: &NodeId, visit: &mut Vec<NodeId>) -> Vec<NodeId> {   
    let mut points_with_ids: Vec<GraphPoint> = Vec::new();
    
    let mut points: Vec<Point> = Vec::new();

    for node_id in visit {
        let point = graph.nodes().get(node_id).unwrap().point();
        points_with_ids.push(GraphPoint { point: *point, id: *node_id  });
        points.push(*point);
    }

    let ls = LineString::from(points);

    // note: concavity factor could be radius dependend
    // note: concave hull is not sorted
    let hull = ls.concave_hull(2.0);

    let mut ring: Vec<Point> = hull.exterior().points().collect();
    ring.dedup();
    let mut inner: Vec<Point> = ls.points().filter(|p| ! ring.contains(p)).collect();
    inner.dedup();
     
    
    let start_point = graph.nodes().get(start).unwrap().point();

    
    let bearing_first = geo::HaversineBearing::haversine_bearing(start_point, *ring.first().unwrap());
    let bearing_last = geo::HaversineBearing::haversine_bearing(start_point, *ring.last().unwrap());

    let quadrant_first = Quadrant::to_quadrant(&bearing_first.to_degrees());
    let mut quadrant_last = Quadrant::to_quadrant(&bearing_last.to_degrees());

    if quadrant_first == quadrant_last {
        // quadrants are identical, choose neighbouring quadrant for return back to start
        let bearing_diff = bearing_first.to_degrees() - bearing_last.to_degrees();

        if bearing_diff < 0.0 {
            // last has a higher bearing than first
            // => (rotate clockwise to the next quadrant)
            quadrant_last = quadrant_last.neighbour_cw();
        }
        else {
            // last has a lower bearing than first
            // => (rotate counterclockwise to the previous quadrant)
            quadrant_last = quadrant_last.neighbour_ccw();
        }
    }
 
    // smallest distance point from start is always first (nearest from start of route)
    let mut before_first: PriorityQueue<NodeId, Reverse<NotNan<f64>>> = PriorityQueue::new();
    // largest distance point from start is always first (nearest to end of ring)
    let mut after_last: PriorityQueue<NodeId, NotNan<f64>> = PriorityQueue::new();
    // note: could be made more efficient by removing the points from inner
    for point in inner {
        let bearing = geo::HaversineBearing::haversine_bearing(start_point, point);
        let quadrant = Quadrant::to_quadrant(&bearing.to_degrees());
        
        if quadrant == quadrant_first || quadrant == quadrant_last {
            // get node ids
            let node_id = back_to_id(&points_with_ids, &point);           
            let distance = Point::euclidean_distance(&start_point, &point);

            if quadrant == quadrant_first {
                before_first.push(node_id, Reverse(NotNan::new(distance).unwrap()));  
            } else  {
                after_last.push(node_id, NotNan::new(distance).unwrap());
            }
        }
    }

    

    let mut result: Vec<NodeId> = Vec::new();
    // result.append(&mut before_first.into_sorted_vec());
    result.append(&mut ring.iter().map(|p| back_to_id(&points_with_ids, p)).collect());
    // result.append(&mut after_last.into_sorted_vec());
     
    result
}

/// used to reidentify Points with their corresponding graph nodes
/// note: symptom of the demand give easy access of Points via references to
///       [GraphNode]s instead of just passing around [NodeId]s
struct GraphPoint {
    point: Point,
    id: NodeId
}
 
fn back_to_id(v: &Vec<GraphPoint>, point: &Point) -> NodeId {
    v.iter().fold(None, |s, gp| {
        match s {
            None => {
                if gp.point == *point { Some(gp.id) }
                else { None }
            },
            Some(id) => Some(id)
        }
    }).unwrap()
}

enum Quadrant {
    NE, SE, SW, NW
}

impl Quadrant {
    pub fn value(&self) -> (u64, u64) { // note: will loose some points (at 0.0, 90.0 ...)
        match *self {
            Self::NE => (0, 90),
            Self::SE => (90, 180),
            Self::SW => (180, 270),
            Self::NW => (270, 360)
        }
    }

    pub fn to_quadrant(degrees: &f64) -> Self {
        if *degrees >= 0.0 && *degrees < 90.0 { return Self::NE; }
        else if *degrees >= 90.0 && *degrees < 180.0 { return Self::SE; }
        else if *degrees >= 180.0 && *degrees < 270.0 { return Self::SW; }
        else { return Self::NW; }
    }

    /// counter clock wise neighbouring quadrant
    pub fn neighbour_ccw(&self) -> Self {
        match *self {
            Self::NE => Self::NW,
            Self::SE => Self::NE,
            Self::SW => Self::SE,
            Self::NW => Self::SW
        }
    }

    // clockwise neighbouring quadrant
    pub fn neighbour_cw(&self) -> Self {
        match *self {
            Self::NE => Self::SE,
            Self::SE => Self::SW,
            Self::SW => Self::NW,
            Self::NW => Self::NE,
        }
    }
}

impl PartialEq for Quadrant {
    fn eq(&self, other: &Self) -> bool {
        self.value().0 == other.value().0
    }
}
