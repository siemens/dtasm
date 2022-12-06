FROM ubuntu:20.04

ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends build-essential ca-certificates git cmake wget curl xxd jupyter \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=1.65.0 -y \
    && rustup target add wasm32-wasi \
    && wget -q -O /tmp/wasi-sdk_16.0_amd64.deb https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-16/wasi-sdk_16.0_amd64.deb \
    && dpkg -i /tmp/wasi-sdk_16.0_amd64.deb \
    && rm /tmp/wasi-sdk_16.0_amd64.deb
