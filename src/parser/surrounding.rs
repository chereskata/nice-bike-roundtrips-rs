use std::collections::HashMap;
use geo::Point;
use osmpbfreader::Node as OsmNode;
use osmpbfreader::Way as OsmWay;

use crate::parser::data::*;

pub fn interesting_surrounding(
    data: &OsmData,
    start: &Point,
    route_distance: &u8
) -> Vec<Point> {
    let radius = assumed_radius(route_distance);

    println!("rad: {:?}", radius);
    
    // let mut points: Vec<Point> = viewpoints(&data.nodes)
        // .iter().filter_map(|id| {
            // let point = into_point(data.nodes.get(&id).unwrap());
            // let distance = geo::algorithm::VincentyDistance::vincenty_distance(start, &point).unwrap_or(f64::MAX);
            // if distance < radius {
                // return Some(point);
            // }
            // None
        // })
        // .collect();

    let mut points: Vec<Point> = Vec::new();

    points.append(&mut center_of_lakes(&data, start, &radius));
    
    points
}

fn viewpoints(data: &OsmData) -> Vec<NodeId> {
    let viewpoint_ids: Vec<NodeId> = data.nodes
        .iter()
        .filter(|(_, node)| node.tags.contains("tourism", "viewpoint"))
        .map(|(id, _)| *id)
        .collect();
    viewpoint_ids
}

fn center_of_lakes(data: &OsmData, start: &Point, radius: &f64) -> Vec<Point> {
    // let mut centroid_of_lakes: Vec<Point> = Vec::new();
    
    data.ways
        .iter()
        .filter_map(|(_, way)| {
            if way.tags.iter().any(|(k, v)| k == "natural" && v == "water") {
                let points: Vec<geo::Coord> = way.nodes
                    .iter()
                    .map(|node_id| {
                        let node = data.nodes.get(&node_id.0.unsigned_abs()).unwrap();
                        into_point(&node).0
                    })
                    .collect();
                // check for bigger lakes only
                // if points.len() < 10 { return None; }
                let poly = geo::Polygon::new(geo::LineString::new(points), vec![]);
                // ls.close();
                let area = geo::algorithm::ChamberlainDuquetteArea::chamberlain_duquette_unsigned_area(&poly);                 
                if area < 100.0 { return None; }

                let centroid = geo::algorithm::Centroid::centroid(&poly).unwrap();
                if geo::algorithm::HaversineDistance::haversine_distance(&centroid, start) > *radius {
                    return None;
                }
                
                return Some(centroid);
            }
            None
        })
        .collect()
}

// return radius in meters - distance is kms 
fn assumed_radius(distance: &u8) -> f64 {
    *distance as f64 * 1000.0
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

        let result = viewpoints(&data);
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

    // #[test]
    // fn do_sued_interesting_points() {
    //     let data = data_from_pbf(
    //         "resources/dortmund_sued.osm.pbf"
    //     );

    //     let result = interesting_points(&data);
    //     let real: Vec<Point> = vec![
    //         Point::new(51.4704933_f64, 7.4506593_f64),
    //         Point::new(51.4765483_f64, 7.4648567_f64),
    //         Point::new(51.4816900_f64, 7.4700769_f64),
    //         Point::new(51.4822454_f64, 7.4689146_f64),
    //         Point::new(51.4824034_f64, 7.4702574_f64),
    //         Point::new(51.4712190_f64, 7.4498130_f64),
    //         Point::new(51.4771334_f64, 7.4640114_f64),
    //     ];

    //     // for p in &real {
    //     //     assert!(result
    //     //         .iter()
    //     //         .any(|candidate| {
    //     //             println!("{} {} {} {}", p.x(), candidate.x(), p.y(), candidate.y());
    //     //             p.x() == candidate.x() && p.y() == candidate.y()
    //     //         }));
    //     // }

    //     assert_eq!(result.len(), real.len());
    // }
}