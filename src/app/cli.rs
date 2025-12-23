use clap::Arg;

pub fn build() -> clap::Command {
    clap::command!()
        .next_help_heading("Positional arguments")
        .args([
            Arg::new("query")
                .required(true)
                .value_name("PATTERN")
                .help("A regular expression query."),
            Arg::new("root").value_name("PATH").long_help(
                "A directory to start searching from. Defaults to the current directory.

Must contain a valid `bsrc.toml` at its root.",
            ),
        ])
        .next_help_heading("Search options")
        .args([
            Arg::new("all")
                .short('a')
                .long("all")
                .required(false)
                .conflicts_with("query")
                .action(clap::ArgAction::SetTrue)
                .help("Display all files"),
            Arg::new("case_sensitive")
                .long("case-sensitive")
                .action(clap::ArgAction::SetTrue)
                .help("Execute query case sensitively"),
        ])
}
