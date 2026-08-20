#!/usr/bin/env python3
"""Phase 0 live spike: Renode PolarFire + GDB + LED JSON.

Exit 0 only if:
  * GDB sees 5 threads
  * thread 2 (U54_1) can break on rust_main
  * LED JSON records a True after continue
  * LED JSON stops changing after interrupt (halt freeze)
"""
from __future__ import annotations

import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path
import shutil

OUT = Path(os.environ.get("HARTLAB_FIDELITY_DIR", "/tmp/hartlab-fidelity"))
# Image root is read-only under lock-down. Renode writes next to the
# .resc/.repl (compiled scripts, analyzer cache). Copy the slice we
# include onto the tmpfs work dir when cwd is not writable.
_SRC_ROOT = Path(__file__).resolve().parents[2]


def work_root() -> Path:
    if os.access(_SRC_ROOT, os.W_OK):
        return _SRC_ROOT
    dest = OUT / "tree"
    if not (dest / "platforms/polarfire/playground.resc").is_file():
        dest.mkdir(parents=True, exist_ok=True)
        shutil.copytree(_SRC_ROOT / "platforms", dest / "platforms", dirs_exist_ok=True)
        shutil.copytree(_SRC_ROOT / "examples", dest / "examples", dirs_exist_ok=True)
    return dest


ROOT = work_root()
ELF = ROOT / "examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky"
RESC = ROOT / "platforms/polarfire/playground.resc"
HW = Path(os.environ.get("HWSTATE_PATH", str(OUT / "hw.jsonl")))
GDB_PORT = int(os.environ.get("HARTLAB_GDB_PORT", "3333"))
RENODE = os.environ.get("RENODE", "renode")
GDB = os.environ.get("RISCV_GDB", "riscv-none-elf-gdb")


def log(msg: str) -> None:
    print(f"[fidelity] {msg}", flush=True)


def wait_port(port: int, timeout: float = 30.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        s = socket.socket()
        s.settimeout(0.3)
        try:
            s.connect(("127.0.0.1", port))
            s.close()
            return
        except OSError:
            time.sleep(0.2)
        finally:
            s.close()
    raise RuntimeError(f"nothing listening on 127.0.0.1:{port} after {timeout}s")


def dump_renode_log() -> None:
    p = OUT / "renode.log"
    if not p.is_file():
        log("no renode.log")
        return
    text = p.read_text(errors="replace")
    lines = text.splitlines()
    hits = [
        i
        for i, ln in enumerate(lines)
        if "Exception" in ln or "Could not" in ln or "error:" in ln.lower()
    ]
    if hits:
        i = hits[0]
        log("renode.log exception:\n" + "\n".join(lines[max(0, i - 2) : i + 20]))
    log("renode.log tail:\n" + "\n".join(lines[-40:]))


def start_renode() -> subprocess.Popen:

    OUT.mkdir(parents=True, exist_ok=True)
    if HW.exists():
        HW.unlink()
    logf = (OUT / "renode.log").open("w")
    env = os.environ.copy()
    env["HWSTATE_PATH"] = str(HW)
    # CWD is the repo so playground.resc `$ROOT?=.` resolves. Renode
    # concatenates every -e with `;` and cannot parse `$ROOT=/abs/path`.
    cmd = [
        RENODE,
        "--disable-xwt",
        "--plain",
        "--pid-file",
        str(OUT / "renode.pid"),
        "-e",
        "include @platforms/polarfire/playground.resc",
    ]
    log("start " + " ".join(cmd))
    # Keep stdin open. --console treats EOF as quit and disposes the machine.
    return subprocess.Popen(
        cmd,
        cwd=str(ROOT),
        stdout=logf,
        stderr=subprocess.STDOUT,
        stdin=subprocess.PIPE,
        env=env,
        start_new_session=True,
    )


def gdb_script(path: Path) -> None:
    path.write_text(
        f"""set pagination off
set confirm off
file {ELF}
target remote 127.0.0.1:{GDB_PORT}
printf "=== THREADS ===\\n"
info threads
printf "=== SELECT U54_1 ===\\n"
thread 2
printf "=== BREAK rust_main ===\\n"
break rust_main
printf "=== MONITOR START ===\\n"
monitor start
printf "=== CONTINUE TO rust_main ===\\n"
continue
printf "=== HIT rust_main ===\\n"
info registers pc
printf "=== FIDELITY_HIT ===\\n"
"""
    )


def run_gdb(script: Path) -> str:
    logf = OUT / "gdb.log"
    cmd = [GDB, "-batch", "-n", "-x", str(script)]
    log("gdb " + " ".join(cmd))
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=45)
        text = (proc.stdout or "") + "\n" + (proc.stderr or "")
    except subprocess.TimeoutExpired as exc:
        text = (exc.stdout or "") + "\n" + (exc.stderr or "")
        text = (text if isinstance(text, str) else text.decode("utf-8", "replace"))
        logf.write_text(text)
        raise
    logf.write_text(text)
    return text


def continue_then_interrupt() -> None:
    """Second GDB: continue so LEDs run, then interrupt to freeze."""
    script = OUT / "gdb-run.gdb"
    script.write_text(
        f"""set pagination off
set confirm off
file {ELF}
target remote 127.0.0.1:{GDB_PORT}
printf "=== CONTINUE FOR LEDS ===\\n"
continue
"""
    )
    log("gdb continue (will interrupt)")
    proc = subprocess.Popen(
        [GDB, "-batch", "-n", "-x", str(script)],
        stdout=(OUT / "gdb-run.log").open("w"),
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
    time.sleep(3.0)
    if proc.poll() is None:
        os.killpg(proc.pid, signal.SIGINT)
        try:
            proc.wait(timeout=8)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait(timeout=3)
    log(f"gdb-run exit {proc.returncode}")


def count_threads(gdb_out: str) -> int:
    n = 0
    for line in gdb_out.splitlines():
        s = line.strip()
        if s[:1].isdigit() and "Thread" in s:
            n += 1
        if s.startswith("*") and "Thread" in s:
            n += 1
    return n


def led_saw_on(path: Path) -> bool:
    if not path.is_file():
        return False
    text = path.read_text()
    return '"leds": [true' in text or "true" in text.lower() and "leds" in text


def led_line_count(path: Path) -> int:
    if not path.is_file():
        return 0
    return sum(1 for line in path.read_text().splitlines() if line.strip())


def stop(proc: subprocess.Popen) -> None:
    if proc.poll() is not None:
        return
    try:
        if proc.stdin:
            proc.stdin.close()
    except OSError:
        pass
    try:
        os.killpg(proc.pid, signal.SIGTERM)
        proc.wait(timeout=5)
    except (ProcessLookupError, subprocess.TimeoutExpired):
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def main() -> int:
    if not ELF.is_file():
        log(f"missing ELF {ELF} — run ./scripts/build-blinky.sh")
        return 2
    if not Path(RENODE).is_file() and not _which(RENODE):
        log(f"missing RENODE={RENODE} — run ./scripts/fetch-host-tools.sh")
        return 2
    if not Path(GDB).is_file() and not _which(GDB):
        log(f"missing RISCV_GDB={GDB}")
        return 2

    OUT.mkdir(parents=True, exist_ok=True)
    renode = start_renode()
    try:
        wait_port(GDB_PORT, timeout=40)
        log("GDB stub is up")
        script = OUT / "gdb-smoke.gdb"
        gdb_script(script)
        out = run_gdb(script)
        threads = count_threads(out)
        log(f"thread lines counted: {threads}")
        if threads < 5:
            log("FAIL: expected 5 GDB threads")
            log(out[-2000:])
            return 1
        if "FIDELITY_HIT" not in out or "rust_main" not in out:
            log("FAIL: did not hit rust_main")
            log(out[-2000:])
            return 1
        log("hit rust_main")

        before = led_line_count(HW)
        continue_then_interrupt()
        after_run = led_line_count(HW)
        time.sleep(1.2)
        after_halt = led_line_count(HW)
        log(f"LED jsonl lines: start={before} after_run={after_run} after_halt={after_halt}")
        if HW.is_file():
            log("hw.jsonl tail:\n" + "\n".join(HW.read_text().splitlines()[-8:]))
        else:
            log("FAIL: no hw.jsonl — playground.py did not emit (LED bind failed?)")
            log((OUT / "renode.log").read_text()[-2500:])
            return 1
        if after_run <= before:
            log("FAIL: no LED events while running")
            return 1
        if not led_saw_on(HW):
            log("FAIL: LED never went true")
            return 1
        if after_halt != after_run:
            log("FAIL: LED JSON still changing after halt")
            return 1
        log("PASS: 5 threads, rust_main, LED on, freeze on halt")
        return 0
    except Exception as exc:
        log(f"FAIL: {exc}")
        dump_renode_log()
        return 1
    finally:
        stop(renode)


def _which(name: str) -> bool:
    from shutil import which

    return which(name) is not None


if __name__ == "__main__":
    sys.exit(main())
