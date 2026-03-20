use crate::{App, Dir};
use regex::Regex;
use ssh2::Sftp;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

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

    /// Set the query.
    ///
    /// # Errors
    ///
    /// Returns an error if the `try_into()` fails when converting to a Regex
    /// instance.
    pub fn query<T>(
        &mut self,
        pat: T,
    ) -> Result<(), <T as std::convert::TryInto<regex::Regex>>::Error>
    where
        T: TryInto<Regex>,
    {
        self.query = pat.try_into()?;
        Ok(())
    }

    /// Set the regex used to clean filenames.
    ///
    /// # Errors
    ///
    /// Returns an error if the `try_into()` fails when converting to a Regex
    /// instance.
    pub fn clean<T>(
        &mut self,
        pat: T,
    ) -> Result<(), <T as std::convert::TryInto<regex::Regex>>::Error>
    where
        T: TryInto<Regex>,
    {
        self.clean = Some(pat.try_into()?);
        Ok(())
    }

    /// Set the regex used to ignore filenames.
    ///
    /// # Errors
    ///
    /// Returns an error if the `try_into()` fails when converting to a Regex
    /// instance.
    pub fn ignore<T>(
        &mut self,
        pat: T,
    ) -> Result<(), <T as std::convert::TryInto<regex::Regex>>::Error>
    where
        T: TryInto<Regex>,
    {
        self.ignore = Some(pat.try_into()?);
        Ok(())
    }

    #[must_use]
    pub fn run(&self, dir: &Dir, client: Option<Arc<Sftp>>) -> Vec<String> {
        client.map_or_else(
            || self.run_local(dir),
            |client| self.run_remote(dir, &client),
        )
    }

    fn run_local(&self, dir: &Dir) -> Vec<String> {
        let mut matches: Vec<String> = Vec::new();

        for entry in dir.path.read_dir().unwrap().filter_map(Result::ok) {
            // Skip this file if it doesn't have the required extension.
            if let Some(ext_req) = &dir.extension
                && entry
                    .path()
                    .extension()
                    .is_none_or(|fe| !fe.eq_ignore_ascii_case(ext_req))
            {
                continue;
            }

            if let Some(filename) =
                Self::matching_filename(self, dir, &entry.path(), entry.path().is_file())
            {
                matches.push(filename);
            }
        }

        matches
    }

    fn run_remote(&self, dir: &Dir, client: &Arc<Sftp>) -> Vec<String> {
        let mut matches: Vec<String> = Vec::new();

        let files: Vec<(PathBuf, ssh2::FileStat)> = client.readdir(&dir.path).unwrap();

        for (path, file_stat) in files {
            // Skip this file if it doesn't have the required extension.
            if let Some(ext_req) = &dir.extension
                && path
                    .extension()
                    .is_none_or(|fe| !fe.eq_ignore_ascii_case(ext_req))
            {
                continue;
            }

            if let Some(filename) = Self::matching_filename(self, dir, &path, file_stat.is_file()) {
                matches.push(filename);
            }
        }

        matches
    }

    /// Returns the (mutated) filename, if the file matches this query.
    ///
    /// `is_file` logic is delegated to the caller because remotely-fetched files
    /// must use their metadata to determine the file type.
    fn matching_filename(&self, dir: &Dir, path: &Path, is_file: bool) -> Option<String> {
        // Surely this won't cause issues later
        let is_dir = !is_file;

        if dir.match_dirs && is_file || !dir.match_dirs && is_dir {
            return None;
        }

        let filename = if is_file {
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
            return None;
        }

        // Apply cleaning based on user-specified pattern. If it exists.
        // e.g. "Pokemon Snap (USA).n64" -> "Pokemon Snap"
        let filename = if let Some(re) = &self.clean {
            re.replace_all(&filename, "")
        } else {
            filename
        };

        if self.query.is_match(unaccent::unaccent(&filename).trim()) {
            Some(filename.trim().to_string())
        } else {
            None
        }
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
