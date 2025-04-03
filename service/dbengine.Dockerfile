# Builds are incremental, i.e., code changes should compile pretty fast.
# To wipe the build caches and do a fresh rebuild of everything: docker builder prune --filter type=exec.cachemount


# 0. Compilation/runtime basics: working C/C++ compiler with support for C++ modules
FROM debian:trixie AS basis
RUN apt-get update && \
    apt-get install -y clang-18 clang-tools-18 lld-18 && \
    apt-get clean && \
    update-alternatives --install /usr/bin/cc cc /usr/bin/clang-18 100 && \
    update-alternatives --install /usr/bin/c++ c++ /usr/bin/clang++-18 100 && \
    update-alternatives --install /usr/bin/ld ld /usr/bin/ld.lld-18 100


# 1. Build C++ component (compiler)
FROM basis AS builder-cpp
RUN apt-get update && \
    apt-get install -y --no-install-recommends cmake default-jre-headless git ninja-build && \
    apt-get clean && \
    mkdir -p /src /build /output

COPY jit /src/

RUN --mount=type=cache,id=cache-cpp,target=/build,sharing=locked \
    cd /build && \
    cmake ../src -G Ninja -DCMAKE_BUILD_TYPE=MinSizeRel -DCMAKE_CXX_COMPILER_CLANG_SCAN_DEPS:FILEPATH=/usr/bin/clang-scan-deps-18 && \
    ninja jit && \
    cp jit *.so /output/


# 2. Build Rust
# TODO switch to trixie as soon as images are available
FROM rust:bookworm AS builder-rust
COPY dbengine /src/dbengine
COPY interface /src/interface
RUN --mount=type=cache,id=cache-rust-engine,target=/src/dbengine/target,sharing=locked \
    --mount=type=cache,id=cache-rust-interface,target=/src/interface/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    mkdir -p /output && \
    cd /src/dbengine && \
    cargo build --release && \
    cp target/release/dbengine /output/


# 3. Build runtime container
FROM basis AS runtime
# install assembler + linker
RUN mkdir -p /app/compiler /app/dbengine

COPY --from=builder-cpp /output/* /app/compiler/
COPY --from=builder-rust /output/* /app/dbengine/

WORKDIR /app/dbengine
STOPSIGNAL SIGINT

CMD ["/app/dbengine/dbengine"]
