use crate::model::{FileState, Snapshot};
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

// Bound subprocess latency and output. Git must never invoke user diff drivers.
fn git(cwd: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().context("missing Git stdout")?;
    let reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_millis(750) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            bail!("Git snapshot timed out");
        }
        thread::sleep(Duration::from_millis(2));
    };
    let bytes = reader
        .join()
        .map_err(|_| anyhow::anyhow!("Git reader failed"))??;
    if bytes.len() > 4 * 1024 * 1024 {
        bail!("Git output exceeded snapshot limit");
    }
    if !status.success() {
        bail!("Git query failed");
    }
    Ok(bytes)
}
fn value(cwd: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git(cwd, args)?)?.trim_end().to_owned())
}
fn fingerprint(path: &Path) -> Option<String> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    if metadata.file_type().is_symlink() {
        return Some(format!("link:{}", std::fs::read_link(path).ok()?.display()));
    }
    if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
        return None;
    }
    let mut file = std::fs::File::open(path).ok()?.take(8 * 1024 * 1024 + 1);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut total = 0;
    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        total += n;
        if total > 8 * 1024 * 1024 {
            return None;
        }
        hasher.update(&buffer[..n]);
    }
    Some(hex(&hasher.finalize()))
}

// Lowercase hex, identical to the `{:x}` digest format stored by earlier versions.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Added/removed line counts; `None` for binary content.
pub type LineStat = Option<(u64, u64)>;

/// Line counts of the worktree against HEAD, for comparison time only.
/// Tracked paths come from `git diff --numstat`; untracked text files count
/// every line as added. Paths without a result (unborn HEAD, oversized or
/// unreadable files) are absent from the map.
pub fn numstat(root: &Path, untracked: &[&str]) -> Result<BTreeMap<String, LineStat>> {
    let mut stats = BTreeMap::new();
    let bytes = git(
        root,
        &[
            "diff",
            "--numstat",
            "-z",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "HEAD",
        ],
    )?;
    for entry in bytes.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        let text = String::from_utf8_lossy(entry);
        let mut fields = text.splitn(3, '\t');
        let (Some(added), Some(removed), Some(path)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let counts = added.parse().ok().zip(removed.parse().ok());
        stats.insert(path.to_owned(), counts);
    }
    for path in untracked {
        let full = root.join(path);
        let Ok(metadata) = std::fs::symlink_metadata(&full) else {
            continue;
        };
        if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
            continue;
        }
        let Ok(content) = std::fs::read(&full) else {
            continue;
        };
        let counts = if content.contains(&0) {
            None
        } else {
            let lines = content.iter().filter(|b| **b == b'\n').count()
                + usize::from(content.last().is_some_and(|b| *b != b'\n'));
            Some((lines as u64, 0))
        };
        stats.insert((*path).to_owned(), counts);
    }
    Ok(stats)
}

pub fn capture(cwd: &Path) -> Snapshot {
    let mut snap = Snapshot {
        cwd: cwd.to_string_lossy().into_owned(),
        repo: None,
        branch: None,
        head: None,
        files: BTreeMap::new(),
        warning: None,
    };
    let root = match value(cwd, &["rev-parse", "--show-toplevel"]) {
        Ok(root) => root,
        Err(_) => {
            snap.warning = Some(
                "No Git snapshot: outside a repository, Git unavailable, or query failed.".into(),
            );
            return snap;
        }
    };
    snap.repo = Some(root.clone());
    snap.branch = value(cwd, &["symbolic-ref", "--quiet", "--short", "HEAD"]).ok();
    snap.head = value(cwd, &["rev-parse", "--verify", "HEAD"]).ok();
    match git(
        Path::new(&root),
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=all",
        ],
    ) {
        Ok(bytes) => {
            let mut entries = bytes.split(|b| *b == 0).filter(|b| !b.is_empty());
            while let Some(entry) = entries.next() {
                if entry.len() < 4 {
                    continue;
                }
                let status = String::from_utf8_lossy(&entry[..2]).to_string();
                let path = match std::str::from_utf8(&entry[3..]) {
                    Ok(path) => path.to_owned(),
                    Err(_) => {
                        snap.warning =
                            Some("Non-UTF-8 paths omitted; comparison is incomplete.".into());
                        continue;
                    }
                };
                let hash = fingerprint(&Path::new(&root).join(&path));
                if hash.is_none() && !status.contains('D') {
                    snap.warning = Some("Some files cannot be fingerprinted (unreadable, non-regular, or over 8 MiB). Comparison is incomplete.".into());
                }
                snap.files.insert(
                    path,
                    FileState {
                        status: status.clone(),
                        fingerprint: hash,
                    },
                );
                if (status.contains('R') || status.contains('C'))
                    && let Some(old) = entries.next()
                {
                    snap.files.insert(
                        String::from_utf8_lossy(old).into_owned(),
                        FileState {
                            status: "D ".into(),
                            fingerprint: None,
                        },
                    );
                }
            }
        }
        Err(error) => snap.warning = Some(error.to_string()),
    }
    snap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_lowercase_sha256_hex() {
        let dir = std::env::temp_dir().join(format!("nivra-fp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("abc.txt");
        std::fs::write(&file, b"abc").unwrap();
        let digest = fingerprint(&file);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(
            digest.as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
    }
}
