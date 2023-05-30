use geo::Point;
use geo::Polygon;
use osmpbfreader::Node as OsmNode;
use osmpbfreader::Way as OsmWay;
use osmpbfreader::Relation as OsmRelation;

use crate::parser::data::*;

// returns some of all interesting points, so route will always different (todo)
pub fn interesting_surrounding(
    data: &OsmData,
    start: &Point,
    travel_distance: &u8
) -> Vec<Point> {
    let radius = assumed_radius(travel_distance);
    
    let mut points: Vec<Point> = Vec::new();

    points.append(&mut data.nodes
        .iter()
        .filter_map(|node| interesting_node(node.1))
        .collect()
    );

    points.append(&mut data.ways
        .iter()
        .filter_map(|way| interesting_way(data, way.1))
        .collect()
    );

    points.append(&mut data.relations
        .iter()
        .filter_map(|relation| interesting_relation(data, relation.1))
        .collect()
    );

    // remove all points that are too far away
    // note: include in append steps above to save runtime
    points = points.into_iter()
        .filter(|p| {
            let distance = geo::HaversineDistance::haversine_distance(start, p);
            distance < radius
        })
        .collect();
    

    // to always get a different route, use only 20 random intersting points
    use rand::thread_rng;
    use rand::seq::SliceRandom;
    points.shuffle(&mut thread_rng());
    // println!("points: {}", points.len());
    points.truncate((radius * 0.005) as usize);
    
    points
}

// returns a point if it is interesting
fn interesting_node(node: &OsmNode) -> Option<Point> {
    for tag in node.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        if k == "tourism" {
            match v {
                "viewpoint" | "alpine_hut" | "attraction" | "picnic_site" =>  {
                    return Some(into_point(&node));
                },
                _ => (),
            }
        }
        if k == "man_made" {
            match v {
                "cairn" | "cross" | "lighthouse" | "mineshaft" | "obelisk" |
                "observatory" | "watermill" | "windmill" => {
                    return Some(into_point(&node));
                }
                _ => (),
            }
        }
        if k == "historic" {
            match v {
                "memorial" | "archaeological_site" | "wayside_cross" |
                "ruins" | "wayside_shrine" | "monument" | "building" |
                "castle" | "heritage" | "chruch" | "fort" | "city_gate" |
                "house" | "wreck" | "cannon" | "aircraft" | "farm" | "tower" |
                "monastery" | "locomotive" | "ship" | "tank" | "railway_car" => {
                    return Some(into_point(&node));
                },
                _ => (),
            }
        }
    }
    
    None
}

// returns a point, that is at the center of the way, if it is intersting
fn interesting_way(data: &OsmData, way: &OsmWay) -> Option<Point> {
    for tag in way.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        if k == "natural" {
            match v {
                "water" | "grassland" | "heath" | "wood" | "bay" |
                "beach" | "coastline" | "dune"  => {
                    let poly = to_polygon(data, way);
                    let area = area(&poly);
                    if area > 100.0 { return Some(center(&poly)); }
                },
                _ => (),
            }
        }
        if k == "landuse" {
            match v {
                "farmland" | "forest" | "flowerbed" | "meadow" | "orchard" |
                "plant_nursery" | "vineyard" | "grass" => {
                    let poly = to_polygon(data, way);
                    let area = area(&poly);
                    if area > 100.0 { return Some(center(&poly)); }
                },
                _ => (),
            }
        }
        if k == "tourism" {
            match v {
                "alpine_hut" | "attraction" | "picnic_site" => {
                    let poly = to_polygon(data, way);
                    return Some(center(&poly));
                },
                _ => (),
            }
        }
        if k == "man_made" {
            match v {
                "cairn" | "obelisk" | "observatory" | "watermill" |
                "windmill" => {
                    let poly = to_polygon(data, way);
                    return Some(center(&poly));
                }
                _ => (),                
            }
        }
        if k == "historic" {
            match v {
                "memorial" | "archaeological_site" | "ruins" |
                "wayside_shrine" | "monument" | "building" |
                "castle" | "heriage" | "church" | "fort" |
                "city_gate" | "house" | "hollow_way" | "wreck" | "aircraft" |
                "farm" | "tower" | "monastery" | "bridge" | "aqueduct" |
                "locomotive" | "ship" | "tank" | "railway_car" => {
                    let poly = to_polygon(data, way);
                    return Some(center(&poly));
                },
                _ => (),
            }
        }
    }
    
    None
}

// returns a point, that is at the center of all ways, if it is interesting
fn interesting_relation(data: &OsmData, relation: &OsmRelation) -> Option<Point> {
    // add heritage

    // check only for multipolygons as of now
    if ! relation.tags.iter().any(|tag| tag.0.as_str() == "type" && tag.1.as_str() == "multipolygon") {
        return None
    }

    for tag in relation.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

              
        if k == "natural" {
            match v {
                "water" | "grassland" | "heath" | "wood" | "bay" |
                "beach" | "coastline" | "dune"  => {
                    return relation_to_point(data, relation);
                },
                _ => (),
            }
        }
        if k == "landuse" {
            match v {
                "farmland" | "forest" | "flowerbed" | "meadow" | "orchard" |
                "plant_nursery" | "vineyard" | "grass" => {
                    return relation_to_point(data, relation);
                },
                _ => (),
            }
        }
    }

    None
}

// computes the center of the multipolygon relation
fn relation_to_point(data: &OsmData, relation: &OsmRelation) -> Option<Point> {
    let mut all_outer_centers: Vec<Point> = relation.refs
        .iter()
        .filter_map(|a_ref| {
            // only outer memberers are interesting for now
            if a_ref.role.as_str() != "outer" { return None; }
            // only ways are intersting for us
            if a_ref.member.is_way() == false { return None; }

            let way_id = a_ref.member.way().unwrap().0.unsigned_abs();
            let way = data.ways.get(&way_id).unwrap();
            
            let poly = to_polygon(data, &way);
            let area = area(&poly);
            if area > 100.0 { center(&poly); }

            None
        })
        .collect();

    use rand::thread_rng;
    use rand::seq::SliceRandom;
    all_outer_centers.shuffle(&mut thread_rng());

    all_outer_centers.pop()
}

// extract a ways coorinates for further processing
fn to_polygon(data: &OsmData, way: &OsmWay) -> Polygon {
    let coords: Vec<geo::Coord> = way.nodes
        .iter()
        .map(|node_id| {
            let node = data.nodes.get(&node_id.0.unsigned_abs()).unwrap();
            into_point(&node).0
        })
        .collect();
    
    Polygon::new(geo::LineString::new(coords), vec![])
}

// area in qm
fn area(poly: &Polygon) -> f64 {    
    geo::algorithm::ChamberlainDuquetteArea::chamberlain_duquette_unsigned_area(poly)
}

// compute the center point (Centroid) of an area
fn center(poly: &Polygon) -> Point {
    geo::algorithm::Centroid::centroid(poly).unwrap()
}

// return radius in meters - distance is kms 
fn assumed_radius(travel_distance: &u8) -> f64 {
    *travel_distance as f64 * 1000.0 / 6.28
}

fn into_point(node: &OsmNode) -> Point {
    Point::new(node.lon(), node.lat())
}

#[cfg(test)]
mod tests {
    // use super::{*, viewpoints};
    // use crate::parser::data_from_pbf;

    // #[test]
    // fn do_sued_viewpoints() {
    //     let data = data_from_pbf(
    //         "resources/dortmund_sued.osm.pbf"
    //     );

    //     let result = viewpoints(&data);
    //     let real: Vec<NodeId> = vec![
    //         283102981,
    //         315382554,
    //         583299252,
    //         583299258,
    //         583299260,
    //         701550528,
    //         3952182019,
    //     ];

    //     for id in &real {
    //         assert!(result.contains(&id));
    //     }        
        
    //     assert_eq!(result.len(), real.len());
    // }

    // #[test]
    // fn do_sued_interesting_points() {
    //     let data = data_from_pbf(
    //         "resources/dortmund_sued.osm.pbf"
    //     );

    //     let result = interesting_points(&data);
    //     let real: Vec<Point> = vec![ // note: lat = y and lon = x
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