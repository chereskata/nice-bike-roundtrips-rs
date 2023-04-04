use std::{fs::File, io::BufReader, process::exit};

use osmpbfreader::{OsmPbfReader, OsmObj, osmformat};

/// Build up a Graph from the OpenStreetMap data
pub fn weave(pbf: File) {
    let mut buf = OsmPbfReader::new(BufReader::new(pbf));
    // Parse all OpenStreetMap nodes and ways. Ignore relations
    // note: Relations can yield useful meta knowledge about intersting paths
    let mut objs: Vec<OsmObj> = buf.iter().filter_map(|r| {
        // Only real OsmObjects shall remain
        match r {
            Ok(obj) => Some(obj),
            Err(e) => {
                eprintln!("{:?}", e);
                None
            },
        }
    }).filter(|obj| ! (matches!(obj, OsmObj::Relation(_)) )).collect();

    let mut bikeable_ways = bikable_ways(&mut objs);
    // objs.iter().for_each(|obj| print_object(&obj));
}

/// Removes all bike routing relevant [OsmObj]s from objs Vector and extracts
/// them into their own one
fn bikable_ways(objs: &mut Vec<OsmObj>) -> Vec<OsmObj> {
    let mut bikable_ways: Vec<OsmObj> = Vec::new();

    // Find all Ways that are bikable (note: it should not contain any nodes)
    let mut i = 0;
    while i < objs.len() {
        let obj: &OsmObj = &objs[i];
        if is_bikeable_way(&obj) {
            bikable_ways.push(objs.remove(i));
        }
        else {
            // objs did not shrink from right to left, so the next element
            // has a higher index
            i += 1;
        }
    }
    
    bikable_ways
}

/// Tries to determine if an OsmObj is routable in a blacklist fashion
/// note: Only Ways are routable, Nodes just give the way coordinates
fn is_bikeable_way(obj: &OsmObj) -> bool {
    let way = match obj {
        OsmObj::Node(_) | OsmObj::Relation(_) => return false,
        OsmObj::Way(way) => way,
    };

    // note: Traversing the tags two times does not seem optimal for me
    // check if the way is a passable highway
    // if ! way.tags.iter().any(|tag| tag.0.as_str() == "highway") { return false; }

    // whitelist legally allowed and passable highways
    let mut is_bikeable = false;
    for tag in way.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        // note: trunks and trunk_links could have cycleway by its side
        if k == "highway" {
            match v {
                "primary" | "primary_link" | "secondary" | "secondary_link" | 
                "tertiary" | "tertiary_link" | "unclassified" | "residential" |
                "living_street" | "service" | "path" | "track" | "bridleway" |
                "cycleway" | "footway" | "pedestrian" => { is_bikeable = true; break; },
                _ => return false,
            }
        }
        
    }
    if ! is_bikeable { return false; }
    
    // check for additional conditions for making a way not bikable
    for tag in way.tags.iter() {
        let k = tag.0.as_str();
        let v = tag.1.as_str();

        if k == "access" && v == "private" { return false; } // note: way could have bicycle=yes
        if k == "bicycle" && v == "no" { return false; }
        if k == "motorroad" && v == "yes" { return false; } // note: way could have cycleway=*
        if k == "tracktype" && v == "grade5" { return false; }
        if k == "smoothness" {
            match v {
                "very_bad" | "horrible" | "very_horrible" | "impassable" => return false,
                _ => (),
            }
        }
        if k == "surface" {
            match v {
                "stepping_stones" | "gravel" | "rock" => return false,
                "pebblestone" | "mud" | "sand" | "woodclips" => return false,
                _ => (),
            }
        }
    }
    
    true
}

/// Print the OsmObj
pub fn print_object(obj: &OsmObj) {
    match obj {
        OsmObj::Node(node) => {
            println!("Node with ID:{} has following tags:", node.id.0);
            for tag in node.tags.iter() {
                println!("    tag:{} => value:{}", tag.0, tag.1);
            }
            println!("Node has following lat:{}, lon:{}", node.lat(), node.lon());
        },
        OsmObj::Way(way) => {
            println!("Way with ID:{} has following tags:", way.id.0);
            for tag in way.tags.iter() {
                println!("    tag:{} => value:{}", tag.0, tag.1); 
            }
        
            println!("It contains the following nodes ({}):", way.nodes.len());
            for node_id in way.nodes.iter() {
                println!("Point with node id: {}", node_id.0);   
            }             
        },
        OsmObj::Relation(_) => {
            eprintln!("Relation found, exiting ...");
            exit(1);
        },
    }
}