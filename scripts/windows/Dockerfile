# Compilação cruzada para Windows (x86_64-pc-windows-gnu) a partir de Linux: MinGW-w64 e NSIS.
# Usado por scripts/windows/build.sh.
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl ca-certificates git file xz-utils pkg-config \
      mingw-w64 nsis clang libclang-dev lld llvm \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.0 \
    && rustup target add x86_64-pc-windows-gnu
# clippy and rustfmt for scripts/cross-clippy.sh (rust-toolchain.toml asks for them).
RUN rustup component add clippy rustfmt
