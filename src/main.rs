use std::{fs::File, process::exit};
use osmpbfreader::{OsmPbfReader, OsmObj};

fn main() {
    let f = File::open("resources/dortmund_sued.osm.pbf").expect("Could not find .pbf file");
    // let f = File::open("resources/linnich.pbf").expect("Could not find .pbf file");

    let mut pbf = OsmPbfReader::new(f);

    // let mut graph: geo::GeometryCollection<geo::LineString> = geo::GeometryCollection::default();
        
    for obj in pbf.iter() {
        let obj = obj.unwrap_or_else(|e| {
            eprintln!("{:?}", e);
            exit(1);
        });

        print_object(&obj);
    }
}

fn print_object(obj: &OsmObj) {
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
        _ => (),
    }
}