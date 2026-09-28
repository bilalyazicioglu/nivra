use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};
use tempfile::TempDir;
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_nivra")
}
fn nivra(dir: &Path, data: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .current_dir(dir)
        .env("NIVRA_DATA_DIR", data)
        .args(args)
        .output()
        .unwrap()
}
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-b", "main"]);
    git(
        dir.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    git(dir.path(), &["config", "user.name", "Nivra Test"]);
    fs::write(dir.path().join("app.txt"), "working\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "baseline"]);
    dir
}
fn events(dir: &Path, data: &Path) -> serde_json::Value {
    let out = nivra(dir, data, &["events", "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn working_to_broken_and_repeated_dirty_edits() {
    let repo = repo();
    let data = TempDir::new().unwrap();
    assert!(
        nivra(
            repo.path(),
            data.path(),
            &["run", "--", "sh", "-c", "exit 0"]
        )
        .status
        .success()
    );
    fs::write(repo.path().join("app.txt"), "first edit\n").unwrap();
    assert!(
        nivra(repo.path(), data.path(), &["mark", "working"])
            .status
            .success()
    );
    fs::write(repo.path().join("app.txt"), "second edit\n").unwrap();
    assert_eq!(
        nivra(
            repo.path(),
            data.path(),
            &["run", "--", "sh", "-c", "exit 7"]
        )
        .status
        .code(),
        Some(7)
    );
    let diff = nivra(
        repo.path(),
        data.path(),
        &["diff", "working", "now", "--json"],
    );
    assert!(diff.status.success());
    let diff: serde_json::Value = serde_json::from_slice(&diff.stdout).unwrap();
    assert_eq!(diff["changed_files"][0]["path"], "app.txt");
    assert_eq!(diff["commands"].as_array().unwrap().len(), 1);
    let events = events(repo.path(), data.path());
    assert_eq!(events[0]["exit_code"], 7);
    assert!(events[0]["after"].is_object());
}
#[test]
fn pause_ignore_and_redact_before_storage() {
    let repo = repo();
    let data = TempDir::new().unwrap();
    for args in [
        vec!["pause"],
        vec!["run", "--", "true"],
        vec!["resume"],
        vec!["ignore-next"],
        vec!["run", "--", "true"],
    ] {
        assert!(nivra(repo.path(), data.path(), &args).status.success());
    }
    assert_eq!(
        events(repo.path(), data.path()).as_array().unwrap().len(),
        0
    );
    assert!(
        nivra(
            repo.path(),
            data.path(),
            &["run", "--", "echo", "TOKEN=do-not-persist-876"]
        )
        .status
        .success()
    );
    assert_eq!(
        events(repo.path(), data.path())[0]["command"],
        "[private command redacted]"
    );
    let bytes = fs::read(data.path().join("nivra.db")).unwrap();
    assert!(
        !bytes
            .windows(b"do-not-persist-876".len())
            .any(|s| s == b"do-not-persist-876")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(data.path().join("nivra.db"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(data.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
#[test]
fn database_failure_does_not_block_command() {
    let dir = TempDir::new().unwrap();
    let bad = dir.path().join("not-a-directory");
    fs::write(&bad, "x").unwrap();
    let output = nivra(
        dir.path(),
        &bad,
        &["run", "--", "sh", "-c", "printf still-runs; exit 13"],
    );
    assert_eq!(output.status.code(), Some(13));
    assert_eq!(output.stdout, b"still-runs");
}
#[test]
fn non_git_missing_mark_and_incomplete_event() {
    let dir = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    assert!(
        nivra(dir.path(), data.path(), &["run", "--", "true"])
            .status
            .success()
    );
    assert!(events(dir.path(), data.path())[0]["before"]["warning"].is_string());
    assert!(
        !nivra(dir.path(), data.path(), &["mark", "working"])
            .status
            .success()
    );
    let repo = repo();
    assert!(
        !nivra(repo.path(), data.path(), &["diff", "missing", "now"])
            .status
            .success()
    );
    let mut child = Command::new(bin())
        .args(["hook", "start", "--session", "test"])
        .env("NIVRA_DATA_DIR", data.path())
        .current_dir(repo.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"sleep 99").unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
    assert!(events(repo.path(), data.path())[0]["exit_code"].is_null());
}
#[test]
fn marks_are_scoped_by_repository() {
    let a = repo();
    let b = repo();
    let data = TempDir::new().unwrap();
    assert!(
        nivra(a.path(), data.path(), &["mark", "working"])
            .status
            .success()
    );
    assert!(
        !nivra(b.path(), data.path(), &["diff", "working"])
            .status
            .success()
    );
}
#[test]
fn strange_paths_and_terminal_escape_are_safe() {
    let repo = repo();
    let data = TempDir::new().unwrap();
    assert!(
        nivra(repo.path(), data.path(), &["mark", "working"])
            .status
            .success()
    );
    fs::write(repo.path().join("line\nbreak\u{1b}[31m.txt"), "x").unwrap();
    let diff = nivra(repo.path(), data.path(), &["diff"]);
    assert!(diff.status.success());
    assert!(!diff.stdout.contains(&27));
    assert!(String::from_utf8_lossy(&diff.stdout).contains("line\\nbreak"));
}
#[test]
fn zsh_hooks_preserve_status_and_fail_open() {
    if Command::new("zsh").arg("--version").output().is_err() {
        return;
    }
    let repo = repo();
    let data = TempDir::new().unwrap();
    let script = format!(
        r#"
        eval "$('{binary}' init zsh)"
        eval "$('{binary}' init zsh)"
        [[ ${{#preexec_functions}} -eq 1 ]] || exit 81
        _nivra_preexec 'false'
        false
        _nivra_precmd
        [[ $? -eq 1 ]] || exit 82
        _NIVRA_BIN=/nonexistent/nivra
        _nivra_preexec 'echo alive'
        print -r -- alive
        _nivra_precmd
    "#,
        binary = bin()
    );
    let output = Command::new("zsh")
        .args(["-f", "-c", &script])
        .current_dir(repo.path())
        .env("NIVRA_DATA_DIR", data.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("alive"));
    assert_eq!(events(repo.path(), data.path())[0]["exit_code"], 1);
}
#[test]
fn locked_database_fails_open() {
    let repo = repo();
    let data = TempDir::new().unwrap();
    assert!(
        nivra(repo.path(), data.path(), &["status"])
            .status
            .success()
    );
    let conn = rusqlite::Connection::open(data.path().join("nivra.db")).unwrap();
    conn.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let out = nivra(
        repo.path(),
        data.path(),
        &["run", "--", "sh", "-c", "echo survives; exit 9"],
    );
    assert_eq!(out.status.code(), Some(9));
    assert_eq!(out.stdout, b"survives\n");
}
#[test]
fn diff_since_clean_mark_reports_commands_line_stats_and_state() {
    let repo = repo();
    let data = TempDir::new().unwrap();
    assert!(
        nivra(repo.path(), data.path(), &["mark", "working"])
            .status
            .success()
    );
    fs::write(repo.path().join("app.txt"), "a\nb\nc\n").unwrap();
    fs::write(repo.path().join("notes.txt"), "one\ntwo\nthree\n").unwrap();
    assert!(
        nivra(
            repo.path(),
            data.path(),
            &["run", "--", "sh", "-c", "exit 0"]
        )
        .status
        .success()
    );
    assert_eq!(
        nivra(
            repo.path(),
            data.path(),
            &["run", "--", "sh", "-c", "exit 1"]
        )
        .status
        .code(),
        Some(1)
    );

    let text = nivra(repo.path(), data.path(), &["diff", "working", "now"]);
    assert!(text.status.success());
    let text = String::from_utf8_lossy(&text.stdout);
    for expected in [
        "Since \"working\"",
        "+ sh -c 'exit 0'",
        "+ sh -c 'exit 1'",
        "exit 1",
        "app.txt",
        "+3 -1",
        "notes.txt",
        "+3 -0",
        "2 files, +6 -1",
        "HEAD unchanged",
        "branch unchanged (main)",
        "not proof of causation",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
    }
    assert!(!text.contains("vs HEAD"), "{text}");

    let json = nivra(
        repo.path(),
        data.path(),
        &["diff", "working", "now", "--json"],
    );
    let json: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(json["head_changed"], false);
    assert_eq!(json["branch_changed"], false);
    let app = json["changed_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == "app.txt")
        .unwrap();
    assert_eq!(app["lines"]["added"], 3);
    assert_eq!(app["lines"]["removed"], 1);
    assert_eq!(app["lines"]["basis"], "since_mark");
}
#[test]
fn diff_labels_line_stats_vs_head_when_file_was_dirty_at_mark() {
    let repo = repo();
    let data = TempDir::new().unwrap();
    fs::write(repo.path().join("app.txt"), "first edit\n").unwrap();
    assert!(
        nivra(repo.path(), data.path(), &["mark", "working"])
            .status
            .success()
    );
    fs::write(repo.path().join("app.txt"), "second edit\n").unwrap();
    let text = nivra(repo.path(), data.path(), &["diff", "working", "now"]);
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("+1 -1"), "{text}");
    assert!(text.contains("vs HEAD"), "{text}");
    let json = nivra(
        repo.path(),
        data.path(),
        &["diff", "working", "now", "--json"],
    );
    let json: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(json["changed_files"][0]["lines"]["basis"], "vs_head");
}
