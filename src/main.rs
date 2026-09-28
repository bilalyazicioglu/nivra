mod git;
mod model;
mod privacy;
mod shell;
mod store;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use model::{Snapshot, now_ms};
use std::{
    collections::BTreeSet,
    io::{IsTerminal, Read},
    process::Command,
};
use store::Store;

#[derive(Parser)]
#[command(
    version,
    about = "See what changed between working and broken.",
    long_about = "Nivra · a local development timeline.\nCapture commands and Git state, mark a working moment, compare it with now."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    /// Print an opt-in zsh integration script
    Init {
        #[arg(value_parser = ["zsh"])]
        shell: String,
    },
    /// Record one command, inheriting its terminal and exit code
    Run {
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    /// Show the most recent observed events
    Events {
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..=10000))]
        limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Save the current Git state under a name (replaces that mark in this repo)
    Mark { name: String },
    /// Compare a saved mark with now or another mark in this repository
    Diff {
        #[arg(default_value = "working")]
        from: String,
        #[arg(default_value = "now")]
        to: String,
        #[arg(long)]
        json: bool,
    },
    /// Pause recording globally on this machine
    Pause,
    /// Resume recording
    Resume,
    /// Skip the next capture across all shells
    IgnoreNext,
    /// Show local recording and storage status
    Status,
    /// Check storage integrity and local prerequisites
    Doctor,
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        action: Hook,
    },
}
#[derive(Subcommand)]
enum Hook {
    Start {
        #[arg(long)]
        session: String,
    },
    End {
        #[arg(long)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        exit_code: i32,
    },
}
fn cwd() -> Result<std::path::PathBuf> {
    Ok(std::env::current_dir()?)
}
// Escape terminal control bytes without obscuring ordinary quotes or Unicode.
fn safe(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
fn title(subtitle: &str) {
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        println!("\x1b[1;38;5;117m  N I V R A\x1b[0m  /  {subtitle}\n");
    } else {
        println!("  N I V R A  /  {subtitle}\n");
    }
}
fn begin(command: &str, session: &str) -> Result<Option<String>> {
    let mut db = Store::open()?;
    if privacy::ignored(command) || !db.should_capture()? {
        return Ok(None);
    }
    let snap = git::capture(&cwd()?);
    Ok(Some(db.begin(session, &privacy::redact(command), &snap)?))
}
fn finish(id: &str, code: i32) -> Result<()> {
    let time = now_ms();
    Store::open()?.finish(id, code, &git::capture(&cwd()?), time)
}
fn run(command: Vec<String>) -> Result<i32> {
    let label = command
        .iter()
        .map(|arg| {
            if arg.chars().any(char::is_whitespace) {
                format!("'{}'", arg.replace('\'', "'\\''"))
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let session = format!("run-{}", uuid::Uuid::new_v4());
    let id = match begin(&label, &session) {
        Ok(id) => id,
        Err(error) => {
            eprintln!("nivra: capture unavailable ({error}); running command.");
            None
        }
    };
    // Child inherits stdin/stdout/stderr. Output is never captured by Nivra.
    let result = Command::new(&command[0])
        .args(&command[1..])
        .env("NIVRA_EXPLICIT_RUN", "1")
        .status();
    let code = match result {
        Ok(status) => {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                status
                    .code()
                    .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
            }
            #[cfg(not(unix))]
            {
                status.code().unwrap_or(1)
            }
        }
        Err(error) => {
            eprintln!("nivra: cannot execute {}: {error}", safe(&command[0]));
            127
        }
    };
    if let Some(id) = id
        && let Err(error) = finish(&id, code)
    {
        eprintln!("nivra: could not finish event: {error}");
    }
    Ok(code)
}
fn compare(from: &str, to: &str, json: bool) -> Result<()> {
    let db = Store::open()?;
    let current = git::capture(&cwd()?);
    let repo = current
        .repo
        .as_ref()
        .context("comparison requires a Git working tree")?;
    let (start, before) = db.get_mark(from, repo)?;
    let (end, after) = if to == "now" {
        (now_ms(), current.clone())
    } else {
        db.get_mark(to, repo)?
    };
    if end < start {
        bail!("destination mark is older than source mark");
    }
    let paths: BTreeSet<_> = before.files.keys().chain(after.files.keys()).collect();
    let changed: Vec<&String> = paths
        .into_iter()
        .filter(|path| before.files.get(*path) != after.files.get(*path))
        .collect();
    // Line counts need the live worktree, so they exist only when comparing with now.
    // See docs/decisions/0002: "since mark" is exact only for files clean at the mark.
    let head_changed = before.head != after.head;
    let lines = if to == "now" && after.head.is_some() {
        let untracked: Vec<&str> = changed
            .iter()
            .filter(|path| after.files.get(**path).is_some_and(|f| f.status == "??"))
            .map(|path| path.as_str())
            .collect();
        git::numstat(std::path::Path::new(repo), &untracked).unwrap_or_default()
    } else {
        Default::default()
    };
    let basis = |path: &str| {
        if !head_changed && !before.files.contains_key(path) {
            "since_mark"
        } else {
            "vs_head"
        }
    };
    let changed_json: Vec<_> = changed.iter().map(|path| {
        let stat = lines.get(*path).map(|counts| match counts {
            Some((added, removed)) => serde_json::json!({"added": added, "removed": removed, "basis": basis(path)}),
            None => serde_json::json!({"binary": true, "basis": basis(path)}),
        });
        serde_json::json!({"path": path, "before": before.files.get(*path), "after": after.files.get(*path), "lines": stat})
    }).collect();
    // Query the interval directly: a busy history must not silently truncate comparisons.
    let mut statement = db.conn.prepare("SELECT command,exit_code FROM events WHERE started_ms>=? AND started_ms<=? AND json_extract(before_json,'$.repo')=? ORDER BY started_ms,rowid")?;
    let events = statement
        .query_map(rusqlite::params![start, end, repo], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<i32>>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"from":from,"to":to,"before":before,"after":after,"changed_files":changed_json,"commands":events,"head_changed":head_changed,"branch_changed":before.branch != after.branch,"causality":"observations only"})
            )?
        );
        return Ok(());
    }
    title(&format!("{} → {}", safe(from), safe(to)));
    if to == "now" {
        println!("  Since \"{}\"  ·  {}\n", safe(from), safe(repo));
    } else {
        println!(
            "  Between \"{}\" and \"{}\"  ·  {}\n",
            safe(from),
            safe(to),
            safe(repo)
        );
    }

    println!("  Commands");
    for (command, exit) in &events {
        let outcome = match exit {
            Some(0) => "✓".to_owned(),
            Some(code) => format!("✕ exit {code}"),
            None => "· incomplete".to_owned(),
        };
        println!("    + {}  {outcome}", safe(command));
    }
    if events.is_empty() {
        println!("    No recorded commands.");
    }

    println!("\n  Git");
    let width = changed
        .iter()
        .map(|p| safe(p).chars().count())
        .max()
        .unwrap_or(0);
    let (mut added_total, mut removed_total, mut any_vs_head) = (0, 0, false);
    for path in &changed {
        let status = match after.files.get(*path) {
            Some(file) => file.status.trim().to_owned(),
            None => "clean".to_owned(),
        };
        let stat = match lines.get(*path) {
            Some(Some((added, removed))) => {
                added_total += added;
                removed_total += removed;
                any_vs_head |= basis(path) == "vs_head";
                let suffix = if basis(path) == "vs_head" {
                    "  (vs HEAD)"
                } else {
                    ""
                };
                format!("+{added} -{removed}{suffix}")
            }
            Some(None) => "binary".to_owned(),
            None => String::new(),
        };
        println!("    {status:>5}  {:<width$}  {stat}", safe(path));
    }
    if changed.is_empty() {
        println!("    No observed worktree changes.");
    } else if !lines.is_empty() {
        let scope = if any_vs_head { "  (vs HEAD)" } else { "" };
        println!(
            "           {} file{}, +{added_total} -{removed_total}{scope}",
            changed.len(),
            if changed.len() == 1 { "" } else { "s" }
        );
    }

    println!("\n  State");
    let short = |head: &Option<String>| {
        head.as_deref()
            .map_or("(unborn)".to_owned(), |h| h.chars().take(8).collect())
    };
    let branch = |b: &Option<String>| safe(b.as_deref().unwrap_or("(detached/unborn)"));
    if head_changed {
        println!(
            "    HEAD {} → {}  · committed changes are not expanded yet",
            short(&before.head),
            short(&after.head)
        );
    } else {
        println!("    HEAD unchanged ({})", short(&after.head));
    }
    if before.branch == after.branch {
        println!("    branch unchanged ({})", branch(&after.branch));
    } else {
        println!(
            "    branch {} → {}",
            branch(&before.branch),
            branch(&after.branch)
        );
    }
    for warning in [before.warning.as_ref(), after.warning.as_ref()]
        .into_iter()
        .flatten()
    {
        println!("\n  Note: {}", safe(warning));
    }
    println!("\n  Observed changes, not proof of causation.\n");
    Ok(())
}
fn main() {
    let cli = Cli::parse();
    match execute(cli) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("nivra: {error:#}");
            std::process::exit(1);
        }
    }
}
fn execute(cli: Cli) -> Result<i32> {
    match cli.command.unwrap_or(Commands::Events {
        limit: 20,
        json: false,
    }) {
        Commands::Init { .. } => print!("{}", shell::init()?),
        Commands::Run { command } => return run(command),
        Commands::Events { limit, json } => {
            let events = Store::open()?.events(limit as usize)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&events)?);
            } else {
                title("development timeline");
                if events.is_empty() {
                    println!("  Nothing recorded yet. Try: nivra run -- cargo test\n");
                }
                for event in events.iter().rev() {
                    let symbol = match event.exit_code {
                        Some(0) => "✓",
                        Some(_) => "✕",
                        None => "·",
                    };
                    let duration = event
                        .finished_ms
                        .map(|end| format!("{}ms", (end - event.started_ms).max(0)))
                        .unwrap_or("incomplete".into());
                    println!(
                        "  {symbol}  {:8}  {:>10}  {}",
                        &event.id[..8],
                        duration,
                        safe(&event.command)
                    );
                    println!(
                        "     {} · {}",
                        safe(event.before.branch.as_deref().unwrap_or("no branch")),
                        safe(&event.before.cwd)
                    );
                }
                println!(
                    "\n  Save a baseline: nivra mark working\n  Inspect changes: nivra diff working now\n"
                );
            }
        }
        Commands::Mark { name } => {
            if name == "now"
                || name.is_empty()
                || name.len() > 64
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_-./".contains(c))
            {
                bail!("use 1–64 letters, digits, _, -, . or /; 'now' is reserved");
            }
            let snap: Snapshot = git::capture(&cwd()?);
            if snap.warning.is_some() {
                bail!(
                    "cannot mark an incomplete Git snapshot: {}",
                    snap.warning.unwrap_or_default()
                );
            }
            Store::open()?.mark(&name, &snap)?;
            println!(
                "  ◆ Marked \"{}\" · {} changed file(s)",
                safe(&name),
                snap.files.len()
            );
        }
        Commands::Diff { from, to, json } => compare(&from, &to, json)?,
        Commands::Pause => {
            Store::open()?.set("paused", true)?;
            println!("Recording paused globally.");
        }
        Commands::Resume => {
            Store::open()?.set("paused", false)?;
            println!("Recording resumed.");
        }
        Commands::IgnoreNext => {
            Store::open()?.set("ignore-next", true)?;
            println!("Next eligible command will not be recorded (global).");
        }
        Commands::Status => {
            let db = Store::open()?;
            title("status");
            println!(
                "  Recording   {}",
                if db.setting("paused")? {
                    "paused"
                } else {
                    "enabled (requires hooks or nivra run)"
                }
            );
            println!(
                "  Database    {}\n  Output      never captured\n  Telemetry   none",
                safe(&db.path.display().to_string())
            );
        }
        Commands::Doctor => {
            let db = Store::open()?;
            let integrity: String = db.conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
            if integrity != "ok" {
                bail!("SQLite integrity check failed");
            }
            let git = Command::new("git")
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success());
            title("doctor");
            println!(
                "  ✓ SQLite integrity\n  ✓ Database opened\n  {} Git available\n  · Shell hook installation: verify with a recorded command\n  · Alpha: synchronous hooks; no latency guarantee",
                if git { "✓" } else { "✕" }
            );
            if !git {
                return Ok(1);
            }
        }
        Commands::Hook { action } => match action {
            Hook::Start { session } => {
                if std::env::var_os("NIVRA_EXPLICIT_RUN").is_some() {
                    return Ok(0);
                }
                let mut command = String::new();
                std::io::stdin()
                    .take(64 * 1024)
                    .read_to_string(&mut command)?;
                if let Some(id) = begin(&command, &session)? {
                    println!("{id}");
                }
            }
            Hook::End { id, exit_code } => finish(&id, exit_code)?,
        },
    }
    Ok(0)
}
