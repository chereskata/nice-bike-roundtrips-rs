use std::collections::HashMap;
use geo::Point;
use osmpbfreader::Node as OsmNode;

use crate::parser::data::*;

pub fn interesting_points(data: &OsmData) -> Vec<Point> {
    let points: Vec<Point> = viewpoints(&data.nodes)
        .iter()
        .map(|id| into_point(data.nodes.get(&id).unwrap()))
        .collect();
    points
}

fn viewpoints(nodes: &HashMap<NodeId, OsmNode>) -> Vec<NodeId> {
    let viewpoint_ids: Vec<NodeId> = nodes
        .iter()
        .filter(|(_, node)| node.tags.contains("tourism", "viewpoint"))
        .map(|(id, _)| *id)
        .collect();
    viewpoint_ids
}

fn into_point(node: &OsmNode) -> Point {
    Point::new(node.lat(), node.lon())
}

#[cfg(test)]
mod tests {
    use super::{*, viewpoints};
    use crate::parser::data_from_pbf;

    #[test]
    fn do_sued_viewpoints() {
        let data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        let result = viewpoints(&data.nodes);
        let real: Vec<NodeId> = vec![
            283102981,
            315382554,
            583299252,
            583299258,
            583299260,
            701550528,
            3952182019,
        ];

        for id in &real {
            assert!(result.contains(&id));
        }        
        
        assert_eq!(result.len(), real.len());
    }

    #[test]
    fn do_sued_interesting_points() {
        let data = data_from_pbf(
            "resources/dortmund_sued.osm.pbf"
        );

        let result = interesting_points(&data);
        let real: Vec<Point> = vec![
            Point::new(51.4704933_f64, 7.4506593_f64),
            Point::new(51.4765483_f64, 7.4648567_f64),
            Point::new(51.4816900_f64, 7.4700769_f64),
            Point::new(51.4822454_f64, 7.4689146_f64),
            Point::new(51.4824034_f64, 7.4702574_f64),
            Point::new(51.4712190_f64, 7.4498130_f64),
            Point::new(51.4771334_f64, 7.4640114_f64),
        ];

        // for p in &real {
        //     assert!(result
        //         .iter()
        //         .any(|candidate| {
        //             println!("{} {} {} {}", p.x(), candidate.x(), p.y(), candidate.y());
        //             p.x() == candidate.x() && p.y() == candidate.y()
        //         }));
        // }

        assert_eq!(result.len(), real.len());
    }
}