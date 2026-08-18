# Phase 0 lock-down image: Renode + RISC-V GDB + blinky + live runner.
# Network is used at *build* time only. Runtime is --network=none.
FROM debian:bookworm-slim

ARG RENODE_VERSION=1.16.1
ARG XPACK_VERSION=15.2.0-1
ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl python3 libicu72 libgssapi-krb5-2 \
        libssl3 zlib1g \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 10000 --create-home --home-dir /home/session session \
    && mkdir -p /opt/hartlab /opt/tools /opt/examples /tmp/hartlab-fidelity \
    && chown -R session:session /home/session /tmp/hartlab-fidelity

WORKDIR /opt/tools
RUN curl -fL --retry 3 \
      -o renode.tgz \
      https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.linux-portable-dotnet.tar.gz \
    && mkdir -p /opt/tools/renode \
    && tar -xzf renode.tgz -C /opt/tools/renode --strip-components=1 \
    && rm renode.tgz \
    && curl -fL --retry 3 \
      -o xpack.tgz \
      https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/download/v${XPACK_VERSION}/xpack-riscv-none-elf-gcc-${XPACK_VERSION}-linux-x64.tar.gz \
    && tar -xzf xpack.tgz -C /opt/tools \
    && rm xpack.tgz

# Repo slice needed to boot the playground (read-only at runtime).
COPY platforms /opt/hartlab/platforms
COPY tests/fidelity /opt/hartlab/tests/fidelity
COPY examples/rust/blinky/src /opt/hartlab/examples/rust/blinky/src
COPY examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky \
     /opt/hartlab/examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky

ENV RENODE=/opt/tools/renode/renode \
    RISCV_GDB=/opt/tools/xpack-riscv-none-elf-gcc-15.2.0-1/bin/riscv-none-elf-gdb \
    HARTLAB_FIDELITY_DIR=/tmp/hartlab-fidelity \
    HWSTATE_PATH=/tmp/hartlab-fidelity/hw.jsonl \
    HOME=/home/session

WORKDIR /opt/hartlab
USER 10000:10000
ENTRYPOINT ["python3", "/opt/hartlab/tests/fidelity/run_live.py"]
