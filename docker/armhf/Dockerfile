FROM debian:bullseye

# Install Rust
RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates git \
        gcc libc-dev libsqlite3-dev:armhf gcc-arm-linux-gnueabihf libc6-dev-armhf-cross \
        build-essential debhelper openssh-client && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.63.0 --profile minimal && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add armv7-unknown-linux-gnueabihf

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=arm-linux-gnueabihf-gcc \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc

LABEL version=4
