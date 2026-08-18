# HartLab session image — STUB. Does not yet install Renode or GDB.
# Do not treat a successful `docker build` as the Phase 0 Docker exit.
# Next real work: bake pinned Renode portable + riscv64-unknown-elf-gdb +
# hartlab-agent, USER 10000, ENTRYPOINT the agent.
FROM debian:bookworm-slim

RUN useradd --uid 10000 --create-home --home-dir /home/session session \
    && mkdir -p /opt/examples /opt/platforms /run/agent \
    && chown -R session:session /home/session /run/agent

COPY platforms/polarfire /opt/platforms/polarfire

USER 10000:10000
WORKDIR /home/session
# Intentionally not a working lab. /bin/true would hide that.
CMD ["echo", "hartlab-session stub: Renode and GDB are not in this image yet"]
