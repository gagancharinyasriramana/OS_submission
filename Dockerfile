FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    grub-pc-bin \
    grub-common \
    xorriso \
    mtools \
    git \
    python3 \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2024-05-01 --profile minimal \
    && rustup component add rust-src llvm-tools-preview

WORKDIR /work

CMD ["bash", "scripts/build_iso.sh"]
