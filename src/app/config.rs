use super::{Dir, parser};
use regex::Regex;
use serde::Deserialize;

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
