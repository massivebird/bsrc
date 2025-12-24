use colored::{ColoredString, Colorize};
use eyre::Context;
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fs::exists, io::Read, path::PathBuf};

mod cli;

#[derive(Debug, Clone)]
pub struct App {
    pub root: PathBuf,
    pub query: Regex,
    pub config: Config,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub dirs: HashMap<String, Dir>,

    #[serde(rename = "clean")]
    raw_clean: Option<String>,
    #[serde(rename = "ignore")]
    raw_ignore: Option<String>,

    #[serde(skip)]
    pub clean: Option<Regex>,
    #[serde(skip)]
    pub ignore: Option<Regex>,
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
    pub fn build() -> Result<Self, eyre::Report> {
        let matches = cli::build().get_matches();

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

            Regex::new(&format!("{opts}{raw_query}"))
                .wrap_err_with(|| "Failed to parse query expression.".to_string())?
        };

        let root = get_arg("root").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

        let toml_path = root.join("bsrc.toml");

        let mut f = std::fs::File::open(&toml_path)
            .wrap_err_with(|| format!("Failed to read config from {}", toml_path.display()))?;

        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();

        let mut config: Config = match toml::from_str(&buf) {
            Ok(c) => c,
            Err(e) => {
                panic!("{e}");
            }
        };

        for dir in config.dirs.values_mut() {
            // Verify that this dir path exists.
            assert!(exists(root.join(dir.path.clone())).is_ok_and(|b| b));

            // Build colored prefixes.
            dir.color_prefix = dir
                .raw_prefix
                .truecolor(dir.color[0], dir.color[1], dir.color[2]);
        }

        // Optionally filter directories.
        if let Some(ids) = get_arg("only") {
            config.dirs = config
                .dirs
                .into_iter()
                .filter(|d| ids.contains(&d.0))
                .collect();
        } else if let Some(ids) = get_arg("exclude") {
            config.dirs = config
                .dirs
                .into_iter()
                .filter(|d| !ids.contains(&d.0))
                .collect();
        }

        // Build clean regex.
        config.clean = {
            match config.raw_clean {
                Some(ref pattern) => Some(
                    Regex::new(&format!("(?i){pattern}"))
                        .wrap_err("Failed to parse `clean` regex pattern from config file.")?,
                ),
                None => None,
            }
        };

        // Build ignore regex.
        config.ignore = {
            match config.raw_ignore {
                Some(ref pattern) => Some(
                    Regex::new(&format!("(?i){pattern}"))
                        .wrap_err("Failed to parse `ignore` regex pattern from config file.")?,
                ),
                None => None,
            }
        };

        Ok(Self {
            root,
            query,
            config,
        })
    }
}
