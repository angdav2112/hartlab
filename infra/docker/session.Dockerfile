# HartLab session image. Pin Renode + GDB versions before any public run.
# Build: docker build -f infra/docker/session.Dockerfile -t hartlab-session:dev .
FROM debian:bookworm-slim

RUN useradd --uid 10000 --create-home --home-dir /home/session session \
    && mkdir -p /opt/examples /opt/platforms /run/agent \
    && chown -R session:session /home/session /run/agent

# Placeholders: copy Renode portable + riscv64-unknown-elf-gdb in a later PR.
COPY platforms/polarfire /opt/platforms/polarfire
COPY apps/agent /opt/agent-src

USER 10000:10000
WORKDIR /home/session
# The real entrypoint is hartlab-agent. Until that binary is baked in:
ENTRYPOINT ["/bin/true"]
