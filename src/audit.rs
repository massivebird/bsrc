use crate::app::Dir;
use ssh2::Sftp;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

/// Audit entry point. Performs an audit and prints any output to the console.
pub async fn report_audit(app: crate::app::App) -> std::io::Result<()> {
    let mut handles: VecDeque<tokio::task::JoinHandle<_>> =
        VecDeque::with_capacity(app.config.dirs.len());

    for dir in app.config.dirs.clone() {
        let remote: Option<_> = app.remote_client.clone();
        handles.push_back(tokio::spawn(async move { audit_bus(&dir, remote) }));
    }

    for dir in app.config.dirs {
        let mut msgs: Vec<String> = handles.pop_front().unwrap().await?;

        msgs.sort_unstable_by_key(|s| s.to_lowercase());

        for m in msgs {
            println!("{}: {m}", dir.color_prefix);
        }
    }

    Ok(())
}

/// Initiates an audit for the archive, which is either:
///
/// (1) Local with no client, or
/// (2) Remote with a remote client.
fn audit_bus(dir: &Dir, client: Option<Arc<Sftp>>) -> Vec<String> {
    client.map_or_else(|| audit_local(dir), |client| audit_remote(dir, &client))
}

/// Uniformly formats an audit message for the filename, pushing it to `msgs`.
fn push_with_desc(msgs: &mut Vec<String>, filename: &OsStr, desc: &str) {
    msgs.push(format!("{desc}: {}", filename.display()));
}

fn audit_local(dir: &Dir) -> Vec<String> {
    let mut msgs: Vec<String> = Vec::new();

    for entry in dir.path.read_dir().unwrap().filter_map(Result::ok) {
        let filename = entry.file_name();

        if entry.file_name().to_string_lossy().starts_with('.') {
            push_with_desc(&mut msgs, &filename, "hidden");
        } else if let Some(ext_req) = &dir.extension
            && entry
                .path()
                .extension()
                .is_none_or(|fe| !fe.eq_ignore_ascii_case(ext_req))
        {
            push_with_desc(&mut msgs, &filename, "non-matched");
        }
    }

    msgs
}

fn audit_remote(dir: &Dir, client: &Arc<Sftp>) -> Vec<String> {
    let mut msgs: Vec<String> = Vec::new();

    let files: Vec<(PathBuf, ssh2::FileStat)> = client.readdir(&dir.path).unwrap();

    for (path, _file_stat) in files {
        let Some(filename) = path.file_name() else {
            eprintln!(
                "{}: failed to read filename from: {}",
                dir.color_prefix,
                path.display()
            );
            continue;
        };

        if filename.to_string_lossy().starts_with('.') {
            push_with_desc(&mut msgs, filename, "hidden");
        } else if let Some(ext_req) = &dir.extension
            && path
                .extension()
                .is_none_or(|fe| !fe.eq_ignore_ascii_case(ext_req))
        {
            push_with_desc(&mut msgs, filename, "non-matched");
        }
    }

    msgs
}
