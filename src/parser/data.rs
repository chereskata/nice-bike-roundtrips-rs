use std::collections::HashMap;

use osmpbfreader::{Node as OsmNode, Way as OsmWay, Relation as OsmRelation};


pub type NodeId = u64;
pub type WayId = u64;
pub type RelationId = u64;

/// Contains the elements of the OpenStreetMap data, separated by type
/// note: as OpenStreetMap does no guarantees towards the uniqueness of ids
/// between types, it could be possible that a node and way share the same id
pub struct OsmData {
    pub nodes: HashMap<NodeId, OsmNode>,
    pub ways: HashMap<WayId, OsmWay>,
    pub relations: HashMap<RelationId, OsmRelation>
}

impl OsmData {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            relations: HashMap::new()
        }
    }
}
