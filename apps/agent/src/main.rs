//! Session-container agent stub. JSON lines on stdin/stdout. No shell.

use hartlab_protocol::{
    decode_host_line, encode_line, gdb_command_denied, AgentToHost, HostToAgent,
};
use std::io::{self, BufRead, Write};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hartlab_agent=info".into()),
        )
        .init();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    emit(&mut stdout, &AgentToHost::Ready);

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        match decode_host_line(&line) {
            Ok(HostToAgent::Shutdown) => break,
            Ok(HostToAgent::Reset) | Ok(HostToAgent::Start { .. }) => {
                emit(&mut stdout, &AgentToHost::Ready);
            }
            Ok(HostToAgent::GdbMi { payload }) => {
                if let Some(cmd) = gdb_command_denied(&payload) {
                    emit(
                        &mut stdout,
                        &AgentToHost::Fatal {
                            message: format!("denied gdb command: {cmd}"),
                        },
                    );
                    continue;
                }
                emit(&mut stdout, &AgentToHost::GdbMi { payload });
            }
            Err(e) => emit(
                &mut stdout,
                &AgentToHost::Fatal {
                    message: format!("bad host message: {e}"),
                },
            ),
        }
    }
}

fn emit(out: &mut impl Write, msg: &AgentToHost) {
    if let Ok(line) = encode_line(msg) {
        let _ = out.write_all(line.as_bytes());
        let _ = out.flush();
    }
}
