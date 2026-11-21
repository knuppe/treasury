# Ambiente de build Linux do Treasury (mesmas dependências do CI: Ubuntu 22.04). Usado por
# scripts/linux/build.sh.
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl ca-certificates git file xz-utils \
      libwebkit2gtk-4.1-dev libsoup-3.0-dev libayatana-appindicator3-dev libudev-dev libhidapi-dev \
      librsvg2-dev libssl-dev libdbus-1-dev pkg-config patchelf \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.99.0
# bindgen (dependências com código C) precisa da libclang.
RUN apt-get update && apt-get install -y --no-install-recommends clang libclang-dev && rm -rf /var/lib/apt/lists/*
