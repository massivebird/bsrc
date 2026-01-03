use colored::{ColoredString, Colorize};
use eyre::{Context, OptionExt};
use regex::Regex;
use serde::Deserialize;
use std::{fs::exists, path::PathBuf};

mod cli;
mod parser;

#[derive(Debug, Clone)]
pub struct App {
    pub root: PathBuf,
    pub query: Regex,
    pub config: Config,
    pub only_counts: bool,
    pub no_count_output: bool,
    pub no_clean: bool,
    pub no_ignore: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(skip)]
    // Directories are deserialized into hashmap entries, with the ID as the key.
    // I sort and collect them into a vec to create deterministic output.
    pub dirs: Vec<Dir>,

    #[serde(default = "parser::default_output_fmt")]
    pub output_fmt: String,

    #[serde(deserialize_with = "parser::deserialize_regex")]
    #[serde(default)]
    pub clean: Option<Regex>,

    #[serde(deserialize_with = "parser::deserialize_regex")]
    #[serde(default)]
    pub ignore: Option<Regex>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Dir {
    pub path: String,

    #[serde(rename = "prefix")]
    pub raw_prefix: String,

    #[serde(default)]
    pub match_dirs: bool,

    #[serde(default = "parser::default_color")]
    #[serde(deserialize_with = "parser::deserialize_hex")]
    pub color: [u8; 3],

    #[serde(skip)]
    pub color_prefix: ColoredString,
    #[serde(skip)]
    pub id: String,
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

        let mut config = parser::from_toml_path(&root)?;

        // Filter non-existent directories.
        config.dirs.retain(|dir| {
            let dir_path = root.join(dir.path.clone());

            if exists(&dir_path).is_ok_and(|ex: bool| ex) {
                true
            } else {
                warn_msg(&format!(
                    "Path for `dirs.{}` does not exist: {}",
                    dir.id,
                    dir_path.display()
                ));
                false
            }
        });

        // Build colored prefixes.
        for dir in &mut config.dirs {
            dir.color_prefix = dir
                .raw_prefix
                .truecolor(dir.color[0], dir.color[1], dir.color[2]);
        }

        // Optionally filter directories.
        if let Some(ids) = get_arg("only") {
            config.dirs.retain(|d| ids.contains(&d.id));
        } else if let Some(ids) = get_arg("exclude") {
            config.dirs.retain(|d| !ids.contains(&d.id));
        }

        Ok(Self {
            root,
            query,
            config,
            only_counts: matches.get_flag("count"),
            no_count_output: matches.get_flag("no_count"),
            no_clean: matches.get_flag("no_clean"),
            no_ignore: matches.get_flag("no_ignore"),
        })
    }
}

pub fn warn_msg(msg: &str) {
    eprintln!("{}: {msg}", "WARN".yellow());
}
