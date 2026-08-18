//! Host ↔ session-container protocol and ELF accept list.

use serde::{Deserialize, Serialize};

pub const MAX_ELF_BYTES: usize = 4 * 1024 * 1024;
pub const EM_RISCV: u16 = 243;
pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;
pub const ET_EXEC: u16 = 2;
pub const PT_INTERP: u32 = 3;
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

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

/// GDB console commands the agent must refuse (no host/guest shell).
pub const GDB_DENYLIST: &[&str] = &[
    "shell", "pipe", "source", "python", "make", "cd", "target", "run", "attach",
    "file", "add-symbol-file", "dump", "restore", "set logging", "set startup-with-shell",
];

pub fn gdb_command_denied(line: &str) -> Option<&'static str> {
    let trimmed = line.trim().trim_start_matches('-').to_ascii_lowercase();
    GDB_DENYLIST
        .iter()
        .copied()
        .find(|cmd| trimmed == *cmd || trimmed.starts_with(&format!("{cmd} ")))
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ElfError {
    #[error("file is empty")]
    Empty,
    #[error("larger than {MAX_ELF_BYTES} bytes")]
    TooLarge,
    #[error("not an ELF file")]
    NotElf,
    #[error("only ELFCLASS64 is accepted")]
    Not64,
    #[error("only little-endian ELF is accepted")]
    NotLe,
    #[error("machine is not RISC-V (EM_RISCV=243)")]
    NotRiscv,
    #[error("ELF type is not ET_EXEC")]
    NotExec,
    #[error("dynamic interpreter present; Linux userspace ELFs are rejected")]
    HasInterpreter,
    #[error("truncated ELF headers")]
    Truncated,
}

/// Host-side allowlist. Call before bind-mounting an upload into a session.
pub fn validate_elf(bytes: &[u8]) -> Result<(), ElfError> {
    if bytes.is_empty() {
        return Err(ElfError::Empty);
    }
    if bytes.len() > MAX_ELF_BYTES {
        return Err(ElfError::TooLarge);
    }
    if bytes.len() < 64 {
        return Err(ElfError::Truncated);
    }
    if bytes[0..4] != ELF_MAGIC {
        return Err(ElfError::NotElf);
    }
    if bytes[4] != ELFCLASS64 {
        return Err(ElfError::Not64);
    }
    if bytes[5] != ELFDATA2LSB {
        return Err(ElfError::NotLe);
    }

    let e_type = u16::from_le_bytes([bytes[16], bytes[17]]);
    let e_machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if e_machine != EM_RISCV {
        return Err(ElfError::NotRiscv);
    }
    if e_type != ET_EXEC {
        return Err(ElfError::NotExec);
    }

    let e_phoff = u64::from_le_bytes(bytes[32..40].try_into().unwrap()) as usize;
    let e_phentsize = u16::from_le_bytes([bytes[54], bytes[55]]) as usize;
    let e_phnum = u16::from_le_bytes([bytes[56], bytes[57]]) as usize;
    if e_phentsize < 56 || e_phnum > 128 {
        return Err(ElfError::Truncated);
    }
    for i in 0..e_phnum {
        let off = e_phoff.saturating_add(i.saturating_mul(e_phentsize));
        if off.saturating_add(4) > bytes.len() {
            return Err(ElfError::Truncated);
        }
        let p_type = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if p_type == PT_INTERP {
            return Err(ElfError::HasInterpreter);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_rv64_exec(with_interp: bool) -> Vec<u8> {
        let mut b = vec![0u8; 128];
        b[0..4].copy_from_slice(&ELF_MAGIC);
        b[4] = ELFCLASS64;
        b[5] = ELFDATA2LSB;
        b[6] = 1;
        b[16..18].copy_from_slice(&ET_EXEC.to_le_bytes());
        b[18..20].copy_from_slice(&EM_RISCV.to_le_bytes());
        b[32..40].copy_from_slice(&64u64.to_le_bytes());
        b[54..56].copy_from_slice(&56u16.to_le_bytes());
        b[56..58].copy_from_slice(&1u16.to_le_bytes());
        if with_interp {
            b[64..68].copy_from_slice(&PT_INTERP.to_le_bytes());
        }
        b
    }

    #[test]
    fn accepts_static_rv64() {
        assert_eq!(validate_elf(&minimal_rv64_exec(false)), Ok(()));
    }

    #[test]
    fn rejects_interp() {
        assert_eq!(validate_elf(&minimal_rv64_exec(true)), Err(ElfError::HasInterpreter));
    }

    #[test]
    fn rejects_x86() {
        let mut b = minimal_rv64_exec(false);
        b[18..20].copy_from_slice(&62u16.to_le_bytes());
        assert_eq!(validate_elf(&b), Err(ElfError::NotRiscv));
    }

    #[test]
    fn rejects_too_big() {
        let b = vec![0; MAX_ELF_BYTES + 1];
        assert_eq!(validate_elf(&b), Err(ElfError::TooLarge));
    }

    #[test]
    fn denylist_catches_shell() {
        assert_eq!(gdb_command_denied("shell ls"), Some("shell"));
        assert_eq!(gdb_command_denied("info threads"), None);
    }

    #[test]
    fn json_roundtrip() {
        let msg = HostToAgent::Start {
            example_id: Some("blinky-rust".into()),
        };
        let line = encode_line(&msg).unwrap();
        assert_eq!(decode_host_line(&line).unwrap(), msg);
    }
}
