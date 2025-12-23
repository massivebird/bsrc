use clap::Arg;
use colored::{ColoredString, Colorize};
use regex::Regex;
use serde::Deserialize;
use std::{io::Read, path::PathBuf};

#[derive(Debug, Clone)]
pub struct App {
    pub root: PathBuf,
    pub query: Regex,
    pub config: Config,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub dirs: Vec<Dir>,

    #[serde(rename = "clean")]
    raw_clean: Option<String>,
    #[serde(skip)]
    pub clean: Option<Regex>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Dir {
    pub path: String,

    #[serde(rename = "prefix")]
    pub raw_prefix: String,

    #[serde(default)]
    pub match_dirs: bool,

    #[serde(default = "default_color")]
    pub color: [u8; 3],

    #[serde(skip)]
    pub color_prefix: ColoredString,
}

const fn default_color() -> [u8; 3] {
    [255, 255, 255]
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

        let mut config: Config = match toml::from_str(&buf) {
            Ok(c) => c,
            Err(e) => {
                panic!("{e}");
            }
        };

        // Build colored prefixes.
        for dir in &mut config.dirs {
            dir.color_prefix = dir
                .raw_prefix
                .truecolor(dir.color[0], dir.color[1], dir.color[2]);
        }

        // Build clean regex.
        config.clean = {
            config.raw_clean.as_ref().map_or_else(
                || None, // No field specified.
                |pattern| {
                    Regex::new(&format!("(?i){pattern}")).map_or_else(
                        |e| {
                            eprintln!("{e}");
                            std::process::exit(1);
                        },
                        Some,
                    )
                },
            )
        };

        Self {
            query,
            config,
            root: PathBuf::from("/home/penguino/game-archive/"),
        }
    }
}
