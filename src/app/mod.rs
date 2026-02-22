use colored::Colorize;
use eyre::{Context, ContextCompat, OptionExt};
use regex::Regex;
use ssh2::{CheckResult, Session};
use std::{
    env,
    fs::exists,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use self::dir::Dir;

mod cli;
pub mod config;
pub mod dir;
pub mod parser;

pub use config::Config;

#[derive(Clone)]
pub struct App {
    pub query: Regex,
    pub config: Config,
    pub only_counts: bool,
    pub no_count_output: bool,
    pub no_clean: bool,
    pub no_ignore: bool,
    pub remote_client: Option<Arc<Mutex<ssh2::Sftp>>>,
}

impl App {
    /// # Errors
    ///
    /// Some error scenarios:
    ///
    /// + Client fails to establish a connection to the remote machine.
    /// + Fails to parse bsrc.toml.
    ///
    /// # Panics
    ///
    /// Will panic if parsing the command line arguments fails.
    pub fn build() -> Result<Self, eyre::Report> {
        let matches = cli::build().get_matches();

        // Generate CLI completions if prompted, then exit.
        if let Some(sub_matches) = matches.subcommand_matches("completions") {
            let shell = sub_matches
                .get_one::<clap_complete_command::Shell>("shell")
                .wrap_err("No way, I failed to get the shell. #freak accident")?;

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

        let remote = if let Some(remote) = get_arg("remote") {
            let (user, host): (&str, &str) = remote.split_once('@').unwrap();

            let sesh: Session = {
                let mut sesh = Session::new().unwrap();
                sesh.set_tcp_stream(TcpStream::connect(format!("{host}:22")).unwrap());
                sesh.handshake().unwrap();

                sesh
            };

            // Reject the host if it's unknown.
            validate_host(&sesh, host)?;

            let private_key: PathBuf = matches.get_one::<PathBuf>("identity").map_or_else(
                || {
                    Path::new(&std::env::var("HOME").unwrap())
                        .join(".ssh")
                        .join("id_rsa")
                },
                std::convert::Into::into,
            );

            sesh.userauth_pubkey_file(user, None, &private_key, None)?;

            Some(Arc::new(Mutex::new(sesh.sftp()?)))
        } else {
            None
        };

        let mut config = parser::from_toml_path(&root, remote.clone())?;

        // Filter non-existent directories.
        config.dirs.retain(|dir| {
            let dir_path = root.join(dir.path.clone());

            if remote.is_some() || exists(&dir_path).is_ok_and(|ex: bool| ex) {
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
            remote_client: remote,
        })
    }
}

/// Returns `Err` if the server does not match one in `known_hosts`.
fn validate_host(sesh: &Session, host: &str) -> Result<(), eyre::Report> {
    let mut known_hosts = sesh.known_hosts().unwrap();

    // Initialize the known hosts with a global known hosts file
    let hosts_path: PathBuf = Path::new(&env::var("HOME").unwrap()).join(".ssh/known_hosts");

    known_hosts
        .read_file(&hosts_path, ssh2::KnownHostFileKind::OpenSSH)
        .unwrap();

    let (key, _key_type) = sesh.host_key().ok_or("Failed to get host key").unwrap();

    // Require that the server is in `known_hosts` and is legit.
    match known_hosts.check(host, key) {
        CheckResult::Match => (),
        _ => {
            return Err(eyre::eyre!(
                "unknown host: failed to find host in {}.",
                hosts_path.display()
            ));
        }
    }

    Ok(())
}

pub fn warn_msg(msg: &str) {
    eprintln!("{}: {msg}", "WARN".yellow());
}
