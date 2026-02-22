use crate::app::{Config, Dir, warn_msg};
use colored::Colorize;
use eyre::Context;
use regex::Regex;
use serde::{Deserialize, Deserializer, de::Error};
use std::{
    borrow::Cow,
    fs::exists,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Deserialize, Clone, Debug)]
struct DirMap {
    dirs: std::collections::HashMap<String, Dir>,
}

/// Reads `bsrc.toml` at the specified directory.
///
/// # Params
///
/// `remote_client` is used to query a remote file system.
/// Use `None` if you are querying local files.
pub fn from_toml_path(
    path: &Path,
    remote_client: Option<Arc<Mutex<ssh2::Sftp>>>,
) -> Result<Config, eyre::Report> {
    let mut buf = String::new();

    // Read the toml contents either remotely or locally.
    if let Some(client) = remote_client {
        let mut toml = client
            .lock()
            .unwrap()
            .open(path.join("bsrc.toml"))
            .expect("Failed to locate bsrc.toml at the specified path.");

        toml.read_to_string(&mut buf)?;
    } else {
        let toml_path: PathBuf = find_toml(path, "bsrc.toml")?;

        let mut f = std::fs::File::open(&toml_path)
            .wrap_err_with(|| format!("Failed to read config from {}", toml_path.display()))?;

        f.read_to_string(&mut buf)?;
    }

    let mut config: Config = toml::from_str(&buf)?;

    // Build directories and populate them in sorted order.
    config.dirs = {
        let mut dirs: Vec<Dir> = Vec::new();

        let dirs_map: DirMap = toml::from_str(&buf)?;

        for (id, mut dir) in dirs_map.dirs {
            dir.id = id;

            // Build colored prefixes.
            dir.color_prefix = dir
                .raw_prefix
                .truecolor(dir.color[0], dir.color[1], dir.color[2]);

            // Make all paths absolute.
            dir.path = path.join(&dir.path);

            dirs.push(dir);
        }

        dirs.sort_by_key(|d| d.id.clone());

        dirs
    };

    Ok(config)
}

/// Returns the path to `bsrc.toml`. Performs limited upward searches if
/// the file isn't immediately present.
///
/// # Errors
///
/// Returns `Err` if the file cannot be found.
pub fn find_toml(root: &Path, filename: &str) -> eyre::Result<PathBuf> {
    let mut root = root;

    if exists(root.join(filename)).is_ok_and(|b| b) {
        return Ok(root.join(filename));
    }

    warn_msg(&format!("Searching for `{filename}` in parent directories..."));

    for _ in 0..4 {
        let maybe_toml = root.join(filename);

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
        "Failed to locate `{filename}` in current or parent directories."
    ))
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

/// Deserializes regex strings into `regex::Regex` instances.
pub(super) fn deserialize_regex<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = Cow::<'de, str>::deserialize(deserializer)?;

    Regex::new(&buf).map_err(serde::de::Error::custom).map(Some)
}

pub(super) const fn default_color() -> [u8; 3] {
    [255, 255, 255]
}

pub(super) fn default_output_fmt() -> String {
    "%p: %f".to_owned()
}

#[derive(Deserialize, Clone, Debug)]
pub struct Preset {
    pub host: String,
    pub user: String,
    pub path: String,
    #[serde(skip)]
    pub id: String,
}

#[derive(Deserialize, Clone, Debug)]
struct PresetMap {
    presets: std::collections::HashMap<String, Preset>,
}

pub fn read_presets(root: &Path) -> Result<Vec<Preset>, eyre::Report> {
    let mut buf = String::new();

    let toml_path: PathBuf = find_toml(root, "presets.toml")?;

    let mut f = std::fs::File::open(&toml_path)
        .wrap_err_with(|| format!("Failed to read config from {}", toml_path.display()))?;

    f.read_to_string(&mut buf)?;

    let presets_map: PresetMap = toml::from_str(&buf)?;

    let mut presets = Vec::with_capacity(presets_map.presets.len());

    for (id, mut p) in presets_map.presets {
        p.id = id;
        presets.push(p);
    }

    Ok(presets)
}
