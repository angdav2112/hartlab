//! Host-side ELF allowlist. Coarse on purpose: refuse anything that is not a
//! static RV64 executable loadable at the playground DDR base.

pub const MAX_ELF_BYTES: usize = 4 * 1024 * 1024;
pub const PLAYGROUND_LOAD_ADDR: u64 = 0x8000_0000;
pub const EM_RISCV: u16 = 243;
pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;
pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16 = 3;
pub const PT_LOAD: u32 = 1;
pub const PT_INTERP: u32 = 3;
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const EHSIZE: usize = 64;
const PHDR64: usize = 56;

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
    #[error("no PT_LOAD segment covers {PLAYGROUND_LOAD_ADDR:#x}")]
    NoPlaygroundLoad,
    #[error("truncated ELF headers")]
    Truncated,
}

/// Call before bind-mounting an upload into a session.
pub fn validate_elf(bytes: &[u8]) -> Result<(), ElfError> {
    if bytes.is_empty() {
        return Err(ElfError::Empty);
    }
    if bytes.len() > MAX_ELF_BYTES {
        return Err(ElfError::TooLarge);
    }
    if bytes.len() < EHSIZE {
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
    if e_type == ET_DYN || e_type != ET_EXEC {
        return Err(ElfError::NotExec);
    }

    let e_phoff = u64_at(bytes, 32)? as usize;
    let e_phentsize = u16_at(bytes, 54)? as usize;
    let e_phnum = u16_at(bytes, 56)? as usize;
    if e_phentsize < PHDR64 || e_phnum == 0 || e_phnum > 128 {
        return Err(ElfError::Truncated);
    }
    let table_end = e_phoff
        .checked_add(e_phnum.checked_mul(e_phentsize).ok_or(ElfError::Truncated)?)
        .ok_or(ElfError::Truncated)?;
    if table_end > bytes.len() {
        return Err(ElfError::Truncated);
    }

    let mut covers_load = false;
    for i in 0..e_phnum {
        let off = e_phoff + i * e_phentsize;
        if off + PHDR64 > bytes.len() {
            return Err(ElfError::Truncated);
        }
        let p_type = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
        if p_type == PT_INTERP {
            return Err(ElfError::HasInterpreter);
        }
        if p_type == PT_LOAD {
            let vaddr = u64::from_le_bytes(bytes[off + 16..off + 24].try_into().unwrap());
            let memsz = u64::from_le_bytes(bytes[off + 40..off + 48].try_into().unwrap());
            let end = vaddr.saturating_add(memsz);
            if vaddr <= PLAYGROUND_LOAD_ADDR && PLAYGROUND_LOAD_ADDR < end {
                covers_load = true;
            }
        }
    }
    if !covers_load {
        return Err(ElfError::NoPlaygroundLoad);
    }
    Ok(())
}

fn u16_at(bytes: &[u8], off: usize) -> Result<u16, ElfError> {
    bytes
        .get(off..off + 2)
        .and_then(|s| s.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or(ElfError::Truncated)
}

fn u64_at(bytes: &[u8], off: usize) -> Result<u64, ElfError> {
    bytes
        .get(off..off + 8)
        .and_then(|s| s.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(ElfError::Truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_rv64_exec(vaddr: u64, with_interp: bool, phnum: u16) -> Vec<u8> {
        let phoff = 64usize;
        let need = phoff + phnum as usize * PHDR64;
        let mut b = vec![0u8; need.max(128)];
        b[0..4].copy_from_slice(&ELF_MAGIC);
        b[4] = ELFCLASS64;
        b[5] = ELFDATA2LSB;
        b[6] = 1;
        b[16..18].copy_from_slice(&ET_EXEC.to_le_bytes());
        b[18..20].copy_from_slice(&EM_RISCV.to_le_bytes());
        b[32..40].copy_from_slice(&(phoff as u64).to_le_bytes());
        b[54..56].copy_from_slice(&(PHDR64 as u16).to_le_bytes());
        b[56..58].copy_from_slice(&phnum.to_le_bytes());
        if phnum >= 1 {
            let off = phoff;
            b[off..off + 4].copy_from_slice(&PT_LOAD.to_le_bytes());
            b[off + 16..off + 24].copy_from_slice(&vaddr.to_le_bytes());
            b[off + 40..off + 48].copy_from_slice(&0x1000u64.to_le_bytes());
        }
        if with_interp && phnum >= 2 {
            let off = phoff + PHDR64;
            b[off..off + 4].copy_from_slice(&PT_INTERP.to_le_bytes());
        }
        b
    }

    #[test]
    fn accepts_static_rv64_at_ddr() {
        assert_eq!(validate_elf(&minimal_rv64_exec(PLAYGROUND_LOAD_ADDR, false, 1)), Ok(()));
    }

    #[test]
    fn rejects_interp() {
        assert_eq!(
            validate_elf(&minimal_rv64_exec(PLAYGROUND_LOAD_ADDR, true, 2)),
            Err(ElfError::HasInterpreter)
        );
    }

    #[test]
    fn rejects_x86() {
        let mut b = minimal_rv64_exec(PLAYGROUND_LOAD_ADDR, false, 1);
        b[18..20].copy_from_slice(&62u16.to_le_bytes());
        assert_eq!(validate_elf(&b), Err(ElfError::NotRiscv));
    }

    #[test]
    fn rejects_et_dyn() {
        let mut b = minimal_rv64_exec(PLAYGROUND_LOAD_ADDR, false, 1);
        b[16..18].copy_from_slice(&ET_DYN.to_le_bytes());
        assert_eq!(validate_elf(&b), Err(ElfError::NotExec));
    }

    #[test]
    fn rejects_wrong_load_address() {
        assert_eq!(
            validate_elf(&minimal_rv64_exec(0x0800_0000, false, 1)),
            Err(ElfError::NoPlaygroundLoad)
        );
    }

    #[test]
    fn rejects_zero_phnum() {
        assert_eq!(
            validate_elf(&minimal_rv64_exec(PLAYGROUND_LOAD_ADDR, false, 0)),
            Err(ElfError::Truncated)
        );
    }

    #[test]
    fn rejects_truncated_phdr_table() {
        let mut b = minimal_rv64_exec(PLAYGROUND_LOAD_ADDR, false, 1);
        b.truncate(70);
        assert_eq!(validate_elf(&b), Err(ElfError::Truncated));
    }

    #[test]
    fn rejects_too_big() {
        let b = vec![0; MAX_ELF_BYTES + 1];
        assert_eq!(validate_elf(&b), Err(ElfError::TooLarge));
    }

    #[test]
    fn accepts_real_blinky_if_built() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky"
        );
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        assert_eq!(validate_elf(&bytes), Ok(()));
    }
}
