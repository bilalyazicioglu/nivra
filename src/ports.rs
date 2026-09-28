use anyhow::{Result, bail};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// One listening TCP socket. Fields Nivra could not observe are `None`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Listener {
    pub port: u16,
    pub address: String,
    pub family: Option<String>,
    pub pid: u32,
    pub process: Option<String>,
    pub cwd: Option<String>,
}

/// Read-only: inspection never signals, stops or restarts a process.
pub fn listening() -> Result<Vec<Listener>> {
    if !cfg!(target_os = "macos") {
        bail!("port inspection is implemented for macOS only in this alpha");
    }
    let sockets = lsof(&["-nP", "+c0", "-iTCP", "-sTCP:LISTEN", "-F", "pctn"])?;
    let mut listeners = parse_listeners(&sockets);
    let pids: BTreeSet<String> = listeners.iter().map(|l| l.pid.to_string()).collect();
    if !pids.is_empty() {
        let joined = pids.into_iter().collect::<Vec<_>>().join(",");
        // A process may exit between the two queries; its cwd stays unknown.
        if let Ok(output) = lsof(&["-a", "-d", "cwd", "-F", "n", "-p", &joined]) {
            let cwds = parse_cwds(&output);
            for listener in &mut listeners {
                listener.cwd = cwds.get(&listener.pid).cloned();
            }
        }
    }
    Ok(listeners)
}

// Bounded like Git queries: a stuck lsof must not hang the terminal.
fn lsof(args: &[&str]) -> Result<String> {
    let mut child = Command::new("lsof")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().expect("piped stdout");
    let reader = thread::spawn(move || {
        let mut text = String::new();
        stdout
            .take(4 * 1024 * 1024)
            .read_to_string(&mut text)
            .map(|_| text)
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(3) {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            bail!("lsof timed out");
        }
        thread::sleep(Duration::from_millis(5));
    };
    let text = reader
        .join()
        .map_err(|_| anyhow::anyhow!("lsof reader failed"))??;
    // lsof exits 1 when nothing matches; that is an empty answer, not a failure.
    if !status.success() && !text.is_empty() {
        bail!("lsof failed");
    }
    Ok(text)
}

/// Parse `lsof -F pctn` output: `p` starts a process, `c` names it, `t` gives
/// the socket family and `n` its local address.
pub fn parse_listeners(output: &str) -> Vec<Listener> {
    let mut found = BTreeSet::new();
    let (mut pid, mut process, mut family) = (None, None, None);
    for line in output.lines() {
        let Some(tag) = line.chars().next() else {
            continue;
        };
        let value = &line[tag.len_utf8()..];
        match tag {
            'p' => {
                pid = value.parse::<u32>().ok();
                process = None;
                family = None;
            }
            'c' => process = Some(value.to_owned()),
            'f' => family = None,
            't' => family = Some(value.to_owned()),
            'n' => {
                let (Some(pid), Some((address, port))) = (pid, split_address(value)) else {
                    continue;
                };
                found.insert(Listener {
                    port,
                    address,
                    family: family.clone(),
                    pid,
                    process: process.clone(),
                    cwd: None,
                });
            }
            _ => {}
        }
    }
    let mut listeners: Vec<_> = found.into_iter().collect();
    listeners.sort_by(|a, b| (a.port, a.pid, &a.address).cmp(&(b.port, b.pid, &b.address)));
    listeners
}

// "*:3000", "127.0.0.1:5432", "[::1]:8080" → (address, port).
fn split_address(name: &str) -> Option<(String, u16)> {
    let name = name.split(" (").next()?;
    let (address, port) = name.rsplit_once(':')?;
    let address = address.trim_start_matches('[').trim_end_matches(']');
    Some((address.to_owned(), port.parse().ok()?))
}

/// Parse `lsof -d cwd -F n` output into pid → working directory.
pub fn parse_cwds(output: &str) -> BTreeMap<u32, String> {
    let mut cwds = BTreeMap::new();
    let mut pid = None;
    for line in output.lines() {
        if let Some(value) = line.strip_prefix('p') {
            pid = value.parse().ok();
        } else if let (Some(value), Some(pid)) = (line.strip_prefix('n'), pid) {
            cwds.insert(pid, value.to_owned());
        }
    }
    cwds
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOCKETS: &str = "p879\ncnode\nf11\ntIPv4\nn*:3000\nf13\ntIPv6\nn*:3000\nf14\ntIPv4\nn*:3000\n\
p812\ncpostgres\nf7\ntIPv6\nn[::1]:5432\nf8\ntIPv4\nn127.0.0.1:5432\n\
p900\ncControl Center\nf10\ntIPv4\nn*:7000\n";

    #[test]
    fn parses_ipv4_ipv6_and_duplicate_sockets() {
        let listeners = parse_listeners(SOCKETS);
        let summary: Vec<_> = listeners
            .iter()
            .map(|l| (l.port, l.pid, l.address.as_str(), l.family.as_deref()))
            .collect();
        assert_eq!(
            summary,
            [
                (3000, 879, "*", Some("IPv4")),
                (3000, 879, "*", Some("IPv6")),
                (5432, 812, "127.0.0.1", Some("IPv4")),
                (5432, 812, "::1", Some("IPv6")),
                (7000, 900, "*", Some("IPv4")),
            ]
        );
        assert_eq!(listeners[4].process.as_deref(), Some("Control Center"));
    }

    #[test]
    fn tolerates_missing_fields_and_garbage() {
        let listeners =
            parse_listeners("n*:1\np\ncx\nn*:2\np42\nnnot-an-address\nn*:99999\nn*:8080\n\n");
        assert_eq!(listeners.len(), 1);
        assert_eq!((listeners[0].port, listeners[0].pid), (8080, 42));
        assert_eq!(listeners[0].process, None);
        assert_eq!(listeners[0].family, None);
    }

    #[test]
    fn maps_cwds_and_leaves_vanished_processes_unknown() {
        let cwds = parse_cwds("p879\nfcwd\nn/Users/dev/Code/foo\np812\nfcwd\nn/\n");
        assert_eq!(
            cwds.get(&879).map(String::as_str),
            Some("/Users/dev/Code/foo")
        );
        assert_eq!(cwds.get(&812).map(String::as_str), Some("/"));
        assert_eq!(cwds.get(&900), None);
    }
}
