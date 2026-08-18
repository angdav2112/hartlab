# GDB on the PolarFire playground

Renode exposes one GDB stub for all five RV64 harts:

```
(gdb) target remote :3333
(gdb) file blinky
(gdb) info threads
(gdb) thread 2          # usually U54_1
(gdb) info registers mhartid
(gdb) break rust_main
(gdb) continue
```

## SoftConsole quirk

`LoadELF` / GDB `load` programs **e51** only. The playground reset macro
copies `e51.PC` onto `u54_1` and parks `u54_2`–`u54_4`. Firmware still
checks `mhartid` and WFI on every hart except 1.

## Halt freezes the board

When GDB stops a core, Renode virtual time stops. LEDs stop changing.
That is the intended teaching moment.

## Commands that are blocked

The session agent refuses `shell`, `pipe`, `source`, `python`, `make`,
`cd`, `target`, `run`, `file` (after the initial load), `dump`, and
friends. There is no shell PTY.
