use std::collections::VecDeque;
use std::path::Path;

use self::app::{App, Dir};

mod app;

#[tokio::main]
async fn main() {
    let app = app::App::build();

    let mut handles = VecDeque::new();

    for system in app.config.dirs.clone() {
        let app = app.clone();
        handles.push_back(tokio::spawn(async move { query_system(&app, system) }));
    }

    let mut num_matches: u32 = 0;

    for system in app.config.dirs {
        let matches = handles.pop_front().unwrap().await.unwrap();

        num_matches += u32::try_from(matches.len()).unwrap();

        for m in matches {
            println!("{} - {m}", system.prefix);
        }
    }

    println!(
        "{num_matches} {noun} found.",
        noun = match num_matches {
            1 => "match",
            _ => "matches",
        }
    );
}

fn query_system(app: &App, dir: Dir) -> Vec<String> {
    let mut matches: Vec<String> = Vec::new();

    let system_path = app.root.join(dir.path);

    for entry in Path::new(&system_path)
        .read_dir()
        .unwrap()
        .filter_map(Result::ok)
    // .filter(is_not_bios_dir)
    {
        if dir.match_dirs && entry.path().is_file() || !dir.match_dirs && entry.path().is_dir() {
            continue;
        }

        let path = &entry.path();

        let filename = {
            let Some(stem) = path.file_stem() else {
                panic!("Failed to parse filename from path: {}", path.display());
            };

            stem.to_string_lossy()
        };

        // TODO: Apply cleaning.
        // e.g. "Pokemon Snap (USA).n64" -> "Pokemon Snap"

        if app.query.is_match(&filename) {
            matches.push(filename.to_string());
        }
    }

    matches
}
