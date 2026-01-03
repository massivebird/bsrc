use eyre::Context;
use regex::Regex;
use serde::{Deserialize, Deserializer, de::Error};
use std::{
    borrow::Cow,
    fs::exists,
    io::Read,
    path::{Path, PathBuf},
};

use crate::app::{Config, Dir, warn_msg};

#[derive(Deserialize, Clone, Debug)]
struct DirMap {
    dirs: std::collections::HashMap<String, Dir>,
}

pub fn from_toml_path(root: &Path) -> Result<Config, eyre::Report> {
    // Full path to the toml config file.
    let toml_path: PathBuf = find_toml_path(root)?;

    let mut f = std::fs::File::open(&toml_path)
        .wrap_err_with(|| format!("Failed to read config from {}", toml_path.display()))?;

    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .wrap_err("Failed to read contents of TOML config file.")?;

    let dirs_map: DirMap = toml::from_str(&buf).unwrap();

    let mut config: Config = match toml::from_str(&buf) {
        Ok(c) => c,
        Err(e) => {
            panic!("{e}");
        }
    };

    // Insert directories in sorted order
    config.dirs = {
        let mut dirs: Vec<Dir> = Vec::new();

        for (id, mut d) in dirs_map.dirs {
            d.id = id;
            dirs.push(d);
        }

        dirs.sort_by_key(|d| d.id.clone());

        dirs
    };

    Ok(config)
}

/// Searches for `bsrc.toml` in root and (some) parent directories.
pub fn find_toml_path(root: &Path) -> eyre::Result<PathBuf> {
    let mut root = root;

    if exists(root.join("bsrc.toml")).is_ok_and(|b| b) {
        return Ok(root.join("bsrc.toml"));
    }

    warn_msg("Searching for bsrc.toml in parent directories...");

    for _ in 0..4 {
        let maybe_toml = root.join("bsrc.toml");

        if exists(&maybe_toml).is_ok_and(|exists| exists) {
            return Ok(maybe_toml);
        }

        if let Some(upwards) = root.parent() {
            root = upwards;
        } else {
            break;
        }
    }

    Err(eyre::eyre!(
        "Failed to locate `bsrc.toml` in current or parent directories."
    ))
}

/// Deserializes regex strings into `regex::Regex` instances.
pub(super) fn deserialize_regex<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Cow::<'de, str>::deserialize(deserializer)?;

    Regex::new(&buf).map_err(serde::de::Error::custom).map(Some)
}

/// Deserializes hex color strings into rgb values.
pub(super) fn deserialize_hex<'de, D>(deserializer: D) -> Result<[u8; 3], D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Cow::<'de, str>::deserialize(deserializer)?;
    let buf = buf.trim_start_matches('#');

    if buf.len() != 6 {
        return Err(serde::de::Error::custom(toml::de::Error::custom(
            "Unexpected hex color format. Example: \"#33AABB\"",
        )));
    }

    Ok([
        u8::from_str_radix(&buf[0..=1], 16).map_err(serde::de::Error::custom)?,
        u8::from_str_radix(&buf[2..=3], 16).map_err(serde::de::Error::custom)?,
        u8::from_str_radix(&buf[4..=5], 16).map_err(serde::de::Error::custom)?,
    ])
}

pub(super) const fn default_color() -> [u8; 3] {
    [255, 255, 255]
}

pub(super) fn default_output_fmt() -> String {
    "%p: %f".to_owned()
}
