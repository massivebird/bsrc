use clap::Arg;

static QUERY_LONG_HELP: &str = "\
A directory to start searching from. Defaults to the current directory.

Must contain a valid `bsrc.toml` at its root.";

static ONLY_LONG_HELP: &str = "\
Only searches directories with the provided IDs. In the TOML config, directory IDs are in each directory header: `[dirs.<id>]`.

Examples:

# Only search directory with ID \"gba\".
bsrc --only gba \"metal\"

# Supports multiple comma-separated IDs.
bsrc --only gba,snes,ds \"metal\"";

static EXCLUDE_LONG_HELP: &str = "\
Excludes specified directories from the search. See `-o/--only`.";

pub fn build() -> clap::Command {
    clap::command!()
        .args_conflicts_with_subcommands(true)
        .subcommand(
            clap::Command::new("completions")
                .about("Generate shell completions")
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_name("shell")
                        .value_parser(
                            clap::builder::EnumValueParser::<clap_complete_command::Shell>::new(),
                        ),
                ),
        )
        .next_help_heading("Positional arguments")
        .args([
            Arg::new("query")
                .value_name("PATTERN")
                .required_unless_present("all")
                .help("A regular expression query."),
            Arg::new("root")
                .value_name("PATH")
                .long_help(QUERY_LONG_HELP),
        ])
        .next_help_heading("Search options")
        .args([
            Arg::new("all")
                .short('a')
                .long("all")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("Display all files")
                .long_help("Displays all files. Equivalent to the regex query \".\""),
            Arg::new("case_sensitive")
                .long("case-sensitive")
                .action(clap::ArgAction::SetTrue)
                .help("Execute query case sensitively"),
            Arg::new("only")
                .short('o')
                .long("only")
                .value_name("IDs")
                .conflicts_with("exclude")
                .help("Only search directories specified by ID.")
                .long_help(ONLY_LONG_HELP),
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .visible_alias("not")
                .conflicts_with("only")
                .value_name("IDs")
                .help("Exclude directories from search, specified by ID.")
                .long_help(EXCLUDE_LONG_HELP),
            Arg::new("no_ignore")
                .long("no-ignore")
                .action(clap::ArgAction::SetTrue)
                .help("Do not apply config's `ignore` pattern."),
            Arg::new("no_clean")
                .long("no-clean")
                .action(clap::ArgAction::SetTrue)
                .help("Do not apply config's `clean` pattern."),
        ])
        .next_help_heading("Output settings")
        .args([Arg::new("no_count")
            .long("no-count")
            // .conflicts_with("count")
            .action(clap::ArgAction::SetTrue)
            .help("Suppress match count message.")])
}
