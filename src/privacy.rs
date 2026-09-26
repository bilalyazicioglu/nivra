use regex::Regex;
use std::sync::LazyLock;

static SENSITIVE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r#"(?i)(authorization|bearer\s|password|passwd|token|secret|api[_-]?key|private[_-]?key|://[^\s/]+:[^\s/@]+@|(?:^|\s)(?:mysql|mariadb)\b.*\s-p\S+)"#
).unwrap()
});

// Conservatively redact the entire command; partial redaction often leaks quoted values.
pub fn redact(command: &str) -> String {
    if SENSITIVE.is_match(command) {
        "[private command redacted]".into()
    } else {
        command.chars().take(8192).collect()
    }
}
pub fn ignored(command: &str) -> bool {
    command.starts_with(' ')
        || ["pass", "security", "ssh-add"].iter().any(|name| {
            command == *name
                || command
                    .strip_prefix(name)
                    .is_some_and(|tail| tail.starts_with(char::is_whitespace))
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secrets_never_survive() {
        for input in [
            "export API_KEY='two words'",
            "curl -H 'Authorization: Bearer abc' x",
            "mysql -psupersecret",
            "curl https://user:pass@example.com",
            "tool --token abc",
        ] {
            assert_eq!(redact(input), "[private command redacted]");
        }
        assert_eq!(redact("cargo test"), "cargo test");
    }
}
