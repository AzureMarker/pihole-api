FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates libc-dev musl-tools git \
        libsqlite3-dev build-essential debhelper dh-systemd openssh-client && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.52.1 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo install cargo-tarpaulin --features vendored-openssl

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=musl-gcc \
    CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=musl-gcc \
    CC_x86_64_unknown_linux_musl=musl-gcc

LABEL version=4
