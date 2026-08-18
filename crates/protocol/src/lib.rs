//! Host ↔ session-container protocol and ELF accept list.

mod elf;
mod gdb_guard;

pub use elf::{validate_elf, ElfError, MAX_ELF_BYTES, PLAYGROUND_LOAD_ADDR};
pub use gdb_guard::{gdb_command_denied, gdb_payload_allowed};

use serde::{Deserialize, Serialize};

/// Commands the control plane sends to the session agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostToAgent {
    Start { example_id: Option<String> },
    GdbMi { payload: String },
    Reset,
    Shutdown,
}

/// Events the session agent sends to the control plane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentToHost {
    Ready,
    GdbMi { payload: String },
    Hw {
        running: bool,
        leds: [bool; 4],
        #[serde(default, skip_serializing_if = "Option::is_none")]
        uart: Option<String>,
        #[serde(default)]
        ts_us: u64,
    },
    Fatal { message: String },
}

pub fn encode_line(value: &impl Serialize) -> Result<String, serde_json::Error> {
    let mut line = serde_json::to_string(value)?;
    line.push('\n');
    Ok(line)
}

pub fn decode_host_line(line: &str) -> Result<HostToAgent, serde_json::Error> {
    serde_json::from_str(line.trim())
}

pub fn decode_agent_line(line: &str) -> Result<AgentToHost, serde_json::Error> {
    serde_json::from_str(line.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let msg = HostToAgent::Start {
            example_id: Some("blinky-rust".into()),
        };
        let line = encode_line(&msg).unwrap();
        assert_eq!(decode_host_line(&line).unwrap(), msg);
    }
}
