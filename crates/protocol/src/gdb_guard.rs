//! GDB payload filter.
//!
//! Default-deny. Unwrap `-interpreter-exec console "…"` so a denylist on the
//! outer MI verb cannot be bypassed. Abbreviations (`py`, `so`, `sh`) and
//! dangerous MI (`-exec-run`, `-file-exec-and-symbols`) are refused.

/// Why a payload was refused. Returned as the denylist token.
pub fn gdb_command_denied(line: &str) -> Option<&'static str> {
    match classify(line) {
        Decision::Allow => None,
        Decision::Deny(reason) => Some(reason),
    }
}

pub fn gdb_payload_allowed(line: &str) -> bool {
    gdb_command_denied(line).is_none()
}

#[derive(Debug, PartialEq, Eq)]
enum Decision {
    Allow,
    Deny(&'static str),
}

fn classify(raw: &str) -> Decision {
    let line = raw.trim();
    if line.is_empty() {
        return Decision::Deny("empty");
    }

    // Token / console may be wrapped for the MI channel.
    if let Some(inner) = extract_interpreter_exec_console(line) {
        return classify_console(&inner);
    }

    if line.starts_with('-') {
        return classify_mi(line);
    }

    classify_console(line)
}

fn extract_interpreter_exec_console(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let key = "interpreter-exec";
    let idx = lower.find(key)?;
    let rest = line[idx + key.len()..].trim_start();
    let rest_l = rest.to_ascii_lowercase();
    let after_console = if rest_l.starts_with("console") {
        rest["console".len()..].trim_start()
    } else if rest_l.starts_with("--console") {
        rest["--console".len()..].trim_start()
    } else {
        return None;
    };
    Some(unquote(after_console))
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 {
        let b = s.as_bytes();
        if (b[0] == b'"' && b[s.len() - 1] == b'"') || (b[0] == b'\'' && b[s.len() - 1] == b'\'') {
            return s[1..s.len() - 1].replace("\\\"", "\"").replace("\\'", "'");
        }
    }
    s.to_string()
}

fn first_token(s: &str) -> &str {
    s.trim().split_whitespace().next().unwrap_or("")
}

fn classify_mi(line: &str) -> Decision {
    let token = first_token(line);
    let verb = token.trim_start_matches('-').to_ascii_lowercase();
    if MI_ALLOW.iter().any(|a| *a == verb) {
        return Decision::Allow;
    }
    if let Some(reason) = MI_DENY.iter().find(|a| verb == **a || verb.starts_with(&format!("{a}-"))) {
        return Decision::Deny(reason);
    }
    Decision::Deny("mi-default-deny")
}

fn classify_console(line: &str) -> Decision {
    let token = first_token(line).trim_start_matches('-').to_ascii_lowercase();
    if token.is_empty() {
        return Decision::Deny("empty");
    }
    if CONSOLE_DENY.iter().any(|d| token == *d) {
        return Decision::Deny(deny_reason(&token));
    }
    // Multi-word denies: `set logging`, `set startup-with-shell`.
    let lower = line.trim().to_ascii_lowercase();
    if lower.starts_with("set logging") {
        return Decision::Deny("set logging");
    }
    if lower.starts_with("set startup-with-shell") {
        return Decision::Deny("set startup-with-shell");
    }
    if CONSOLE_ALLOW.iter().any(|a| token == *a) {
        return Decision::Allow;
    }
    Decision::Deny("console-default-deny")
}

fn deny_reason(token: &str) -> &'static str {
    CONSOLE_DENY
        .iter()
        .find(|d| **d == token)
        .copied()
        .unwrap_or("console-default-deny")
}

const MI_ALLOW: &[&str] = &[
    "exec-continue",
    "exec-step",
    "exec-next",
    "exec-finish",
    "exec-interrupt",
    "exec-step-instruction",
    "exec-next-instruction",
    "break-insert",
    "break-delete",
    "break-list",
    "break-disable",
    "break-enable",
    "thread-info",
    "thread-select",
    "stack-list-frames",
    "stack-info-frame",
    "stack-list-variables",
    "stack-list-locals",
    "data-list-register-values",
    "data-list-register-names",
    "data-evaluate-expression",
    "data-read-memory-bytes",
    "data-disassemble",
    "file-list-exec-source-file",
    "file-list-exec-source-files",
    "symbol-list-lines",
    "gdb-version",
    "gdb-set",
    "gdb-show",
];

const MI_DENY: &[&str] = &[
    "exec-run",
    "exec-until",
    "file-exec-and-symbols",
    "file-symbol-file",
    "target-select",
    "target-attach",
    "target-detach",
    "target-download",
    "interpreter-exec", // must have been unwrapped already; leftover = deny
];

const CONSOLE_ALLOW: &[&str] = &[
    "info", "i", "thread", "tp", "break", "b", "tbreak", "delete", "d", "disable", "enable",
    "condition", "clear", "continue", "c", "fg", "step", "s", "stepi", "si", "next", "n",
    "nexti", "ni", "finish", "fin", "until", "bt", "backtrace", "where", "frame", "f", "up",
    "down", "print", "p", "x", "disassemble", "disas", "list", "l", "help", "h", "set", "unset",
    "show", "whatis", "ptype", "display", "undisplay",
];

const CONSOLE_DENY: &[&str] = &[
    "shell", "sh", "pipe", "source", "so", "python", "py", "make", "cd", "target", "run", "r",
    "attach", "file", "add-symbol-file", "dump", "restore", "monitor", "compile", "eval",
    "quit", "q", "kill", "detach",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_teaching_commands() {
        for line in [
            "info threads",
            "thread 2",
            "break rust_main",
            "continue",
            "c",
            "step",
            "-exec-continue",
            "-thread-info",
            "-break-insert rust_main",
        ] {
            assert_eq!(gdb_command_denied(line), None, "{line}");
        }
    }

    #[test]
    fn unwraps_interpreter_exec_shell() {
        assert_eq!(
            gdb_command_denied(r#"-interpreter-exec console "shell ls""#),
            Some("shell")
        );
        assert_eq!(
            gdb_command_denied(r#"interpreter-exec console "python import os""#),
            Some("python")
        );
        assert_eq!(gdb_command_denied("py print(1)"), Some("py"));
        assert_eq!(gdb_command_denied("so /tmp/x"), Some("so"));
    }

    #[test]
    fn denies_dangerous_mi() {
        assert_eq!(gdb_command_denied("-exec-run"), Some("exec-run"));
        assert_eq!(
            gdb_command_denied("-file-exec-and-symbols /tmp/x"),
            Some("file-exec-and-symbols")
        );
        assert_eq!(gdb_command_denied("monitor start"), Some("monitor"));
        assert_eq!(gdb_command_denied("unknown-command"), Some("console-default-deny"));
    }
}
