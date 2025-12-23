use clap::Arg;
use regex::Regex;
use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct App {
    pub root: PathBuf,
    pub query: Regex,
    pub config: Config,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub dirs: Vec<Dir>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Dir {
    pub display_name: String,
    pub match_dirs: bool,
    pub color: [u8; 3],
    pub path: String,
}

impl App {
    pub fn build() -> Self {
        let matches = clap::command!()
            .args([
                Arg::new("query")
                    .required(true)
                    .hide(true)
                    .help("Regular expression query"),
                Arg::new("all")
                    .short('a')
                    .long("all")
                    .required(false)
                    .conflicts_with("query")
                    .action(clap::ArgAction::SetTrue)
                    .help("Display all files"),
                Arg::new("case_sensitive")
                    .long("case-sensitive")
                    .action(clap::ArgAction::SetTrue)
                    .help("Execute query case sensitively"),
            ])
            .get_matches();

        // Shortcut for retrieving a command line argument.
        let get_arg = |arg_name: &str| -> Option<&String> { matches.get_one::<String>(arg_name) };

        let query: Regex = {
            let raw_query = if matches.get_flag("all") {
                "."
            } else {
                get_arg("query").unwrap()
            };

            // Default to case-insensitive
            let opts = if matches.get_flag("case_sensitive") {
                ""
            } else {
                "(?i)"
            };

            Regex::new(&format!("{opts}{raw_query}")).unwrap_or_else(|e| {
                eprintln!("{e}");
                std::process::exit(1);
            })
        };

        let mut f = std::fs::File::open("./bsrc.toml").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let config: Config = match toml::from_str(&buf) {
            Ok(c) => c,
            Err(e) => {
                panic!("{e}");
            }
        };

        Self {
            query,
            config,
            root: PathBuf::from("/home/penguino/game-archive/"),
        }
    }
}
