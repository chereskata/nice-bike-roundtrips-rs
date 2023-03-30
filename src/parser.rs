use std::{fs::File, io::BufReader, process::exit};

use osmpbfreader::{OsmPbfReader, OsmObj};

/// Build up a Graph from the OpenStreetMap data
pub fn weave(pbf: File) {
    let mut buf = OsmPbfReader::new(BufReader::new(pbf));
    // Parse all OpenStreetMap nodes and ways. Ignore relations
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

    let mut traversable_objs = traversable_objs(&mut objs);
    // objs.iter().for_each(|obj| print_object(&obj));
}

/// Removes all routable OsmObj from objs Vector and extracts them into their
/// own one
fn traversable_objs(objs: &mut Vec<OsmObj>) -> Vec<OsmObj> {
    let mut traversable_objs: Vec<OsmObj> = Vec::new();

    // Find all OsmObjs that are highways (! for now incl. motorways)
    let mut i = 0;
    while i < objs.len() {
        let obj: &OsmObj = &objs[i];
        if obj.tags().keys().filter(|k| k.as_str() == "highway").count() == 1 {
            traversable_objs.push(objs.remove(i));
        }
        else {
            // objs did not shrink from right to left, so the next element
            // has a higher index
            i += 1;
        }
    }

    // For now traversable_objs should contain only ways
    // but: it also contains nodes with highway=traffic_signs f.e.
    traversable_objs.iter().for_each(|obj| print_object(&obj));
    
    traversable_objs
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