use bsrc::Query;
use regex::Regex;
use std::collections::VecDeque;

mod app;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = bsrc::app::build()?;

    // Holds async task handles, one for each directory.
    // Iterate through `dirs` twice, in the same direction:
    // (1) Spawn an async task for each dir. Push each handle to the back.
    // (2) Pop the front handle when its task is completed.
    // This way, we know exactly which dir corresponds to which handle.
    let mut handles: VecDeque<tokio::task::JoinHandle<_>> =
        VecDeque::with_capacity(app.config.dirs.len());

    let query = Query::from(&app);

    for dir in app.config.dirs.clone() {
        let query: Query = query.clone();
        let remote: Option<_> = app.remote_client.clone();
        handles.push_back(tokio::spawn(async move { query.run(&dir, remote) }));
    }

    let mut total_matches = 0u32;

    // Locates placeholders in user-provided output format string.
    let fmt_re = Regex::new(r"%[pf]")?;

    for dir in app.config.dirs {
        let mut matches: Vec<String> = handles.pop_front().unwrap().await?;

        matches.sort_unstable_by_key(|s| s.to_lowercase());

        total_matches += u32::try_from(matches.len())?;

        if app.only_counts {
            println!("{}:{}", dir.color_prefix, matches.len());
            continue;
        }

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

    if app.no_count_output || app.only_counts {
        println!("{total_matches}");
        return Ok(());
    }

    println!(
        "{total_matches} {noun} found.",
        noun = match total_matches {
            1 => "result",
            _ => "results",
        }
    );

    Ok(())
}
