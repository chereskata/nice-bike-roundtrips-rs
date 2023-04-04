use std::{u8, collections::HashMap};

pub type NodeId = u64;
pub type EdgeId = u64;
/// A graphs node
pub struct Node {
    id: NodeId,
    /// Coordinate data storage and for calculation of geometric properties 
    /// with [geo] crate
    point: geo::Point,
    /// OpenStreetMap tags
    tags: osmpbfreader::Tags,
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
        tags: osmpbfreader::Tags
    ) -> Self {
        Self {
            id,
            point,
            tags,
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
        &self.tags
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

pub struct Edge {
    id: EdgeId,
    /// OpenStreetMap tags
    tags: osmpbfreader::Tags,    
    /// Section length between the Edge's two nodes
    distance: f64,
    /// If true, the edge goes from s-->t and not reachable from t-//->s 
    directed: bool,
    /// if directed, s is the "from" node s-->t
    s: NodeId,
    /// if directed, t is the "to" node s-->t
    t: NodeId,
}

impl Edge {
    /// Create new Edge
    pub fn new(
        id: EdgeId,
        tags: osmpbfreader::Tags,
        distance: f64,
        directed: bool,
        s: NodeId,
        t: NodeId
    ) -> Self {
        Self {
            id,
            tags,
            distance,
            directed,
            s,
            t
        }    
    }
    
    /// OpenStreetMap tags
    pub fn tags(&self) -> &osmpbfreader::Tags {
        &self.tags
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
        &self.s
    }
    /// if directed, t is the "to" node s-->t
    pub fn t(&self) -> &NodeId {
        &self.t
    }
}

pub struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>
}

impl Graph {
    /// Create empty graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new()
        }
    }

    pub fn add_edge(&self, edge: Edge) -> Result<(), String> {
        if ! self.nodes.keys().any(|k| edge.s == *k) {
            return Err(format!(
                "Adding edge #{} failed, it's node #{} is not in this graph!",
                edge.id,
                edge.s
            ));
        }

        if ! self.nodes.keys().any(|k| edge.s == *k) {
            return Err(format!(
                "Adding edge #{} failed, it's node #{} is not in this graph!",
                edge.id,
                edge.s
            ));
        }

        todo!();
    }

    pub fn add_node(&self, node: Node) -> Result<(), String> {
        todo!()
    }
}