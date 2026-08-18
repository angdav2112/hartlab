# Ada blinky — source exhibit, not a gallery ELF

This is a language-parallel **sketch**. It does not import `Interfaces`,
has no crt0 / hart-park, and has no linker script. Do **not** check it
off as a second curated ELF until there is a prebuilt `riscv64` binary
that `validate_elf` accepts.

Same intended contract as `examples/rust/blinky` once that ELF exists
(`platforms/polarfire/MEMORY.md`).

MMIO map matches `platforms/polarfire/MEMORY.md`:

- U54_1 only (`mhartid = 1`)
- SYSREG clock/reset for GPIO2
- GPIO2 pin 16 (Icicle LED1), 0.5 s toggle via CLINT `mtime`

A GNAT `riscv64-elf` / Alire `zfp` runtime is **not** required to work on
this repo today. If you have one:

```text
alr build
```

Otherwise keep this source as the language-parallel reference and attach a
prebuilt ELF in CI later. HartLab’s public playground loads ELFs; it does
not compile Ada on the session host.
