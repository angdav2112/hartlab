# HartLab security notes

Untrusted ELF upload is the threat. Assume Renode or GDB can be crashed
or confused by a malformed ELF.

## Session container

- `--network=none` — no veth, no DNS, no egress
- `--read-only` + tmpfs scratch (`noexec`)
- `--cap-drop ALL` + `no-new-privileges`
- uid 10000, pids 64, 1 CPU, 1536 MB
- image has no compiler, no package manager, no sshd
- agent is the only process the host talks to

## Host

- Axum is not root. Docker socket is not in the session image.
- ELF allowlist (`hartlab_protocol::validate_elf`) before bind-mount
- 15 minute TTL, 5 minute idle, `docker rm -f` is idempotent
- 1 session per IP, hard concurrency cap
- no user URLs fetched (no SSRF)

## GDB

MI only. Console lines go through `-interpreter-exec console`.
Denylist in the agent, not just the UI.

gVisor (`runsc`) is an optional later layer. It is not required to
launch.
