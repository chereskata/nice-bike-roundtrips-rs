use std::{u8, collections::HashMap};

pub type NodeId = u64;
pub type EdgeId = u64;

/// A graphs node
pub struct Node {
    /// Identifies exactly one node, identical to OSM NodeId
    id: NodeId,
    /// Coordinate data storage and for calculation of geometric properties 
    /// with [geo] crate
    point: geo::Point,
    /// Some OSM Ways can be unidirectional (think one-way streets)
    edges: Vec<EdgeId>,
    /// Greatness factor from 0 (industrial zone / unrated)
    /// till 255 (best surroundings imaginable)
    /// note: May be outsourced to hexagonal grid in future
    greatness: u8,
}

impl Node {
    /// Initialize new Node at location [point] and with the following
    /// OpenStreetMap [tags]
    pub fn new(
        id: NodeId,
        point: geo::Point,
    ) -> Self {
        Self {
            id,
            point,
            edges: Vec::new(),
            greatness: 0
        }
    }
    /// Coordinate data storage and for calculation of geometric properties
    /// with [geo] crate    
    pub fn point(&self) -> &geo::Point {
        &self.point
    }
    /// OpenStreetMap tags
    pub fn tags(&self) -> &osmpbfreader::Tags {
        todo!();
    }
    /// Some OSM Ways can be unidirectional (think one-way streets)
    pub fn edges(&self) -> &Vec<EdgeId> {
        &self.edges
    }
    /// Greatness factor from 0 (industrial zone / unrated)
    /// till 255 (best surroundings imaginable)
    /// note: May be outsourced to hexagonal grid in future
    pub fn greatness(&self) -> &u8 {
        &self.greatness
    }
}

/// An edge consists of multiple nodes: Two intersection nodes (s, t) and
/// zero or more shape defining (middle) nodes.
///
/// Every edge starts and ends at an intersection
/// 
/// Dead ends shall be an edge with s=t and directed=true 
pub struct Edge {
    id: EdgeId,
    /// Complete distance from s via all intermediary nodes to t
    distance: f64,
    /// If true, the edge goes from s-->t and not reachable from t-//->s 
    directed: bool,
    /// sorted listing of nodes n.first --> ... --> n.last
    nodes: Vec<NodeId>
}

impl Edge {
    /// Create new Edge
    /// note: maybe compute distance while creating edge ???
    pub fn new(
        id: EdgeId,
        distance: f64,
        directed: bool,
        nodes: Vec<NodeId>
    ) -> Self {
        Self {
            id,
            distance,
            directed,
            nodes
        }    
    }
    
    /// OpenStreetMap tags
    pub fn tags(&self) -> &osmpbfreader::Tags {
        todo!();
    }
    /// Section length between the Edge's two nodes
    pub fn distance(&self) -> &f64 {
        &self.distance
    }
    /// If true, the edge goes from s-->t and not reachable from t-//->s 
    pub fn directed(&self) -> &bool {
        &self.directed
    }
    /// if directed, s is the "from" node s-->t
    pub fn s(&self) -> &NodeId {
        &self.nodes.first().unwrap()
    }
    /// if directed, t is the "to" node s-->t
    pub fn t(&self) -> &NodeId {
        &self.nodes.last().unwrap()
    }
    // get the intermediary nodes with no intersections
    pub fn intermediary(&self) -> &Vec<NodeId> {
        todo!();
    }
    // all nodes
    pub fn nodes(&self) -> &Vec<NodeId> {
        &self.nodes
    }
}

pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>
}

impl Graph {
    /// Create empty graph
    pub fn new(
        nodes: HashMap<NodeId, Node>,
        edges: HashMap<EdgeId, Edge>
    ) -> Self {
        Self {
            nodes,
            edges
        }
    }
    
    pub fn nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    pub fn edges(&self) -> &HashMap<NodeId, Edge> {
        &self.edges
    }
}