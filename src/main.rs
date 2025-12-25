use self::app::{App, Dir};
use regex::Regex;
use std::{collections::VecDeque, path::Path};

mod app;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = app::App::build()?;

    let mut handles = VecDeque::new();

    for (_id, dir) in app.config.dirs.clone() {
        let app = app.clone();
        handles.push_back(tokio::spawn(async move { query_dir(&app, dir) }));
    }

    let mut num_matches: u32 = 0;

    let fmt_re = Regex::new(r"%[pf]").unwrap();

    for dir in app.config.dirs.values() {
        let matches = handles.pop_front().unwrap().await.unwrap();

        num_matches += u32::try_from(matches.len()).unwrap();

        for m in matches {
            // Replace all placeholders in the output format string with their
            // appropriate values.
            let output =
                fmt_re.replace_all(app.config.output_fmt.as_ref(), |caps: &regex::Captures| {
                    match &caps[0] {
                        "%p" => dir.color_prefix.to_string(),
                        "%f" => m.clone(),
                        _ => String::new(),
                    }
                });

            println!("{output}");
        }
    }

    if app.no_count_output {
        return Ok(());
    }

    println!(
        "{num_matches} {noun} found.",
        noun = match num_matches {
            1 => "match",
            _ => "matches",
        }
    );

    Ok(())
}

fn query_dir(app: &App, dir: Dir) -> Vec<String> {
    let mut matches: Vec<String> = Vec::new();

    for entry in Path::new(&app.root.join(dir.path))
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
    {
        if dir.match_dirs && entry.path().is_file() || !dir.match_dirs && entry.path().is_dir() {
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

        if !app.no_ignore
            && app
                .config
                .ignore
                .as_ref()
                .is_some_and(|re| re.is_match(&filename))
        {
            continue;
        }

        // Apply cleaning based on user-specified pattern.
        // e.g. "Pokemon Snap (USA).n64" -> "Pokemon Snap"
        let filename = if let Some(re) = &app.config.clean
            && !app.no_clean
        {
            re.replace_all(&filename, "")
        } else {
            filename
        };

        if app.query.is_match(&filename) {
            matches.push(filename.trim().to_string());
        }
    }

    matches
}
