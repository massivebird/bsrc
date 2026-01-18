use colored::Colorize;
use eyre::{Context, OptionExt};
use regex::Regex;
use std::{fs::exists, path::PathBuf};

use self::dir::Dir;

mod cli;
pub mod config;
pub mod dir;
pub mod parser;

pub use config::Config;

#[derive(Debug, Clone)]
pub struct App {
    pub query: Regex,
    pub config: Config,
    pub only_counts: bool,
    pub no_count_output: bool,
    pub no_clean: bool,
    pub no_ignore: bool,
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

        // Optionally filter directories.
        if let Some(ids) = get_arg("only") {
            config.dirs.retain(|d| ids.contains(&d.id));
        } else if let Some(ids) = get_arg("exclude") {
            config.dirs.retain(|d| !ids.contains(&d.id));
        }

        Ok(Self {
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
