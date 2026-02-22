use colored::Colorize;
use eyre::{ContextCompat, OptionExt};
use regex::Regex;
use ssh2::{CheckResult, Session};
use std::{
    env,
    fs::exists,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use self::{dir::Dir, parser::Preset};

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

            Regex::new(&format!("{opts}{raw_query}"))?
        };

        let preset = if let Some(preset_id) = get_arg("preset") {
            Some(
                parser::read_presets(Path::new("/home/penguino/.config/bsrc/"))?
                    .iter()
                    .find(|p| p.id == *preset_id)
                    .cloned()
                    .wrap_err(eyre::eyre!("no such preset: [presets.{preset_id}]"))?,
            )
        } else {
            None
        };

        let root: PathBuf = if let Some(ref preset) = preset {
            preset.path.clone().into()
        } else if let Some(query) = get_arg("query")
            && matches.get_flag("all")
        {
            // Trying to handle `--all` with a path argument, where the path is
            // stored in the `query` positional argument. Copy it over.
            PathBuf::from(query)
        } else {
            get_arg("root").map_or(std::env::current_dir()?, PathBuf::from)
        };

        let remote: Option<_> = build_remote(preset.as_ref(), get_arg("remote"), &matches)?;

        let mut config: Config = parser::from_toml_path(&root, remote.clone())?;

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
fn reject_host_if_unknown(sesh: &Session, host: &str) -> Result<(), eyre::Report> {
    let mut known_hosts = sesh.known_hosts()?;

    // Initialize the known hosts with a global known hosts file
    let hosts_path: PathBuf = Path::new(&env::var("HOME")?).join(".ssh/known_hosts");

    known_hosts.read_file(&hosts_path, ssh2::KnownHostFileKind::OpenSSH)?;

    let (key, _key_type) = sesh
        .host_key()
        .wrap_err("failed to fetch key from remote server")?;

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

fn build_remote(
    preset: Option<&Preset>,
    remote: Option<&String>,
    matches: &clap::ArgMatches,
) -> Result<Option<Arc<Mutex<ssh2::Sftp>>>, eyre::Report> {
    if preset.is_none() && remote.is_none() {
        return Ok(None);
    }

    let (user, host) = if let Some(preset) = preset {
        (&preset.user[..], &preset.host[..])
    } else if let Some(remote) = remote {
        remote
            .split_once('@')
            .wrap_err("unexpected remote address format. Example: user@hostname")?
    } else {
        unreachable!();
    };

    let sesh: Session = {
        let mut sesh = Session::new()?;
        sesh.set_tcp_stream(TcpStream::connect(format!("{host}:22"))?);
        sesh.handshake()?;

        sesh
    };

    reject_host_if_unknown(&sesh, host)?;

    let private_key: PathBuf = if let Some(path) = matches.get_one::<PathBuf>("identity") {
        path.into()
    } else {
        Path::new(&std::env::var("HOME")?)
            .join(".ssh")
            .join("id_rsa")
    };

    sesh.userauth_pubkey_file(user, None, &private_key, None)?;

    Ok(Some(Arc::new(Mutex::new(sesh.sftp()?))))
}
