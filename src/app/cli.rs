use clap::Arg;

pub fn build() -> clap::Command {
    clap::command!().args([
        Arg::new("query")
            .required(true)
            .hide(true)
            .value_name("PATTERN")
            .help("Regular expression query"),
        Arg::new("root")
            .value_name("PATH")
            .hide(false)
            .help("Query root location"),
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
