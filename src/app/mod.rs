use colored::{ColoredString, Colorize};
use eyre::{Context, OptionExt};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fs::exists, io::Read, path::PathBuf};

mod cli;

#[derive(Debug, Clone)]
pub struct App {
    pub root: PathBuf,
    pub query: Regex,
    pub config: Config,
    pub no_count_output: bool,
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

        // Generate CLI completions if prompted, then exit.
        if let Some(sub_matches) = matches.subcommand_matches("completions") {
            let shell = sub_matches
                .get_one::<clap_complete_command::Shell>("shell")
                .unwrap();

            let mut cli = cli::build();

            shell.generate(&mut cli, &mut std::io::stdout());

            std::process::exit(0);
        }

        // Shortcut for retrieving a command line argument.
        let get_arg = |arg_name: &str| -> Option<&String> { matches.get_one::<String>(arg_name) };

        let query: Regex = {
            let raw_query = if matches.get_flag("all") {
                "."
            } else {
                get_arg("query")
                    .ok_or_eyre("Internal error: failed to retrieve `query` argument.")?
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

        // Trying to handle `--all` with a path argument, where the path is
        // stored in the `query` positional argument. Copy it over.
        let root = if matches.get_flag("all") && get_arg("query").is_some() {
            PathBuf::from(get_arg("query").unwrap())
        } else {
            get_arg("root").map_or(
                std::env::current_dir().wrap_err("Failed to retrieve current directory.")?,
                PathBuf::from,
            )
        };

        let toml_path = root.join("bsrc.toml");

        let mut f = std::fs::File::open(&toml_path)
            .wrap_err_with(|| format!("Failed to read config from {}", toml_path.display()))?;

        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .wrap_err("Failed to read contents of TOML config file.")?;

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

        // Parses a regex field from the TOML config.
        // The `toml` crate doesn't know what regex is, so this is how I
        // convert the string into a regex object.
        macro_rules! get_regex_field {
            ($internal: ident, $external: ident) => {{
                match config.$internal {
                    Some(ref pattern) => {
                        Some(Regex::new(&format!("(?i){pattern}")).wrap_err(format!(
                            "Failed to parse `{}` regex pattern from config file.",
                            stringify!($external)
                        ))?)
                    }
                    None => None,
                }
            }};
        }

        config.clean = if matches.get_flag("no_clean") {
            None
        } else {
            get_regex_field!(raw_clean, clean)
        };

        config.ignore = if matches.get_flag("no_ignore") {
            None
        } else {
            get_regex_field!(raw_ignore, ignore)
        };

        Ok(Self {
            root,
            query,
            config,
            no_count_output: matches.get_flag("no_count"),
        })
    }
}
