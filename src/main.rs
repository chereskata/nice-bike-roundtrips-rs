use std::fs::File;

fn main() {
    let f = File::open("resources/config.toml").unwrap();
    let config = nice_bike_roundtrips::Config::from(f);
    nice_bike_roundtrips::run(config);
}