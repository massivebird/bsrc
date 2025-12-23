use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;

#[derive(Deserialize, Debug)]
struct Config {
    dirs: HashMap<String, Dir>,
}

#[derive(Deserialize, Debug)]
struct Dir {
    match_dirs: bool,
}

fn main() {
    let mut f = std::fs::File::open("./bsrc.toml").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let config: Dir = toml::from_str(&buf).unwrap();
    dbg!(config);
}
