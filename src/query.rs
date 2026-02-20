use crate::{App, Dir};
use regex::Regex;
use remotefs_ssh::SftpFs;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Query {
    pub query: Regex,
    pub clean: Option<Regex>,
    pub ignore: Option<Regex>,
    pub no_clean: bool,
    pub no_ignore: bool,
}

// Just here for convenience
impl Default for Query {
    fn default() -> Self {
        Self {
            query: Regex::new("").unwrap(),
            clean: Option::default(),
            ignore: Option::default(),
            no_clean: Default::default(),
            no_ignore: Default::default(),
        }
    }
}

impl Query {
    #[must_use]
    pub fn new(query: Regex) -> Self {
        Self {
            query,
            ..Default::default()
        }
    }

    pub fn clean(&mut self, regex: Regex) {
        self.clean = Some(regex);
    }

    pub fn ignore(&mut self, regex: Regex) {
        self.ignore = Some(regex);
    }

    #[must_use]
    pub fn run(&self, dir: &Dir, client: Option<Arc<Mutex<SftpFs>>>) -> Vec<String> {
        client.map_or_else(
            || self.run_local(dir),
            |client| self.run_remote(dir, &client),
        )
    }

    fn run_remote(&self, dir: &Dir, client: &Arc<Mutex<SftpFs>>) -> Vec<String> {
        use remotefs::RemoteFs;

        let mut matches: Vec<String> = Vec::new();

        let files: Vec<remotefs::File> = client.lock().unwrap().list_dir(&dir.path).unwrap();

        for file in files {
            let is_file = file.metadata().is_file();
            let is_dir = file.metadata().is_dir();

            if dir.match_dirs && is_file || !dir.match_dirs && is_dir {
                continue;
            }

            let filename = if is_file {
                let Some(stem) = Path::new(file.path()).file_stem() else {
                    panic!(
                        "Failed to parse filename from path: {}",
                        file.path().display()
                    );
                };

                stem.to_string_lossy()
            } else {
                file.path().file_name().unwrap().to_string_lossy()
            };

            // Apply ignore pattern if it exists.
            if self
                .ignore
                .as_ref()
                .is_some_and(|re| re.is_match(&filename))
            {
                continue;
            }

            // Apply cleaning based on user-specified pattern. If it exists.
            // e.g. "Pokemon Snap (USA).n64" -> "Pokemon Snap"
            let filename = if let Some(re) = &self.clean {
                re.replace_all(&filename, "")
            } else {
                filename
            };

            if self.query.is_match(&filename) {
                matches.push(filename.trim().to_string());
            }
        }

        matches
    }

    fn run_local(&self, dir: &Dir) -> Vec<String> {
        let mut matches: Vec<String> = Vec::new();

        for entry in dir.path.read_dir().unwrap().filter_map(Result::ok) {
            if dir.match_dirs && entry.path().is_file() || !dir.match_dirs && entry.path().is_dir()
            {
                continue;
            }

            let path = &entry.path();

            let filename = if entry.path().is_file() {
                let Some(stem) = path.file_stem() else {
                    panic!("Failed to parse filename from path: {}", path.display());
                };

                stem.to_string_lossy()
            } else {
                path.file_name().unwrap().to_string_lossy()
            };

            // Apply ignore pattern if it exists.
            if self
                .ignore
                .as_ref()
                .is_some_and(|re| re.is_match(&filename))
            {
                continue;
            }

            // Apply cleaning based on user-specified pattern. If it exists.
            // e.g. "Pokemon Snap (USA).n64" -> "Pokemon Snap"
            let filename = if let Some(re) = &self.clean {
                re.replace_all(&filename, "")
            } else {
                filename
            };

            if self.query.is_match(&filename) {
                matches.push(filename.trim().to_string());
            }
        }

        matches
    }
}

impl From<&App> for Query {
    fn from(value: &App) -> Self {
        let mut query = Self::new(value.query.clone());

        if let Some(pat) = &value.config.ignore
            && !value.no_ignore
        {
            query.ignore(pat.clone());
        }

        if let Some(pat) = &value.config.clean
            && !value.no_clean
        {
            query.clean(pat.clone());
        }

        query
    }
}
