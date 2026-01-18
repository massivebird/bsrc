use crate::{App, Dir};
use regex::Regex;

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

    pub fn run(&self, dir: &Dir) -> Vec<String> {
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

impl From<App> for Query {
    fn from(value: App) -> Self {
        let mut query = Self::new(value.query);

        if let Some(pat) = value.config.ignore
            && !value.no_ignore
        {
            query.ignore(pat);
        }

        if let Some(pat) = value.config.clean
            && !value.no_clean
        {
            query.clean(pat);
        }

        query
    }
}
