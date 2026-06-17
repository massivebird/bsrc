use crate::app::config::parser;
use colored::ColoredString;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Dir {
    /// The directory's absolute path.
    pub path: std::path::PathBuf,

    #[serde(rename = "prefix")]
    pub raw_prefix: String,

    #[serde(default)]
    pub match_dirs: bool,

    #[serde(default = "parser::default_color")]
    #[serde(deserialize_with = "parser::deserialize_hex")]
    pub color: [u8; 3],

    #[serde(default)]
    pub extension: Option<String>,

    #[serde(skip)]
    pub color_prefix: ColoredString,
    #[serde(skip)]
    pub id: String,
}
