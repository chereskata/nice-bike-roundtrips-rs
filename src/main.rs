use std::{fs::File, process::exit};
use osmpbfreader::{OsmPbfReader, OsmObj};

fn main() {
    // let f = File::open("resources/dortmund_sued.osm.pbf").expect("Could not find .pbf file");
    // let f = File::open("resources/linnich.pbf").expect("Could not find .pbf file");

    // let mut pbf = OsmPbfReader::new(f);

    // // let mut graph: geo::GeometryCollection<geo::LineString> = geo::GeometryCollection::default();
        
    // for obj in pbf.iter() {
    //     let obj = obj.unwrap_or_else(|e| {
    //         eprintln!("{:?}", e);
    //         exit(1);
    //     });

    //     print_object(&obj);
    // }

    nice_bike_roundtrips::run(nice_bike_roundtrips::Config {});
}