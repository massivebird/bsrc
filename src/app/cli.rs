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
            Arg::new("only")
                .short('o')
                .long("only")
                .value_name("IDs")
                .help("Only search directories specified by ID.")
                .long_help("Only searches directories with the provided IDs. In the TOML config, directory IDs are in each directory header: `[dirs.<id>]`.

Examples:

# Only search directory with ID \"gba\".
bsrc --only gba \"metal\"

# Supports multiple comma-separated IDs.
bsrc --only gba,snes,ds \"metal\""),
        ])
}
