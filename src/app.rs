use clap::Arg;
use regex::Regex;
use serde::Deserialize;
use std::io::Read;

#[derive(Debug)]
pub struct App {
    pub query: Regex,
    pub config: Config,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub dirs: Vec<Dir>,
}

#[derive(Deserialize, Debug)]
pub struct Dir {
    pub display_name: String,
    pub match_dirs: bool,
    pub color: [u8; 3],
    pub path: String,
}

impl App {
    pub fn build() -> Self {
        let matches = clap::command!()
            .arg(
                Arg::new("query")
                    .required(true)
                    .hide(true)
                    .help("Regular expression query"),
            )
            .get_matches();

        // Shortcut for retrieving a command line argument.
        let get_arg = |arg_name: &str| -> Option<&String> { matches.get_one::<String>(arg_name) };

        let query = Regex::new(get_arg("query").unwrap()).unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });

        let mut f = std::fs::File::open("./bsrc.toml").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let config: Config = match toml::from_str(&buf) {
            Ok(c) => c,
            Err(e) => {
                panic!("{e}");
            }
        };

        dbg!(&query);

        Self { query, config }
    }
}
