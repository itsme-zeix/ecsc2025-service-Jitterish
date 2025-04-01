# Builds are incremental, i.e., code changes should compile pretty fast.
# To wipe the build caches and do a fresh rebuild of everything: docker builder prune --filter type=exec.cachemount


# 1. Build Rust Application
# TODO switch to trixie as soon as images are available
FROM rust:bookworm AS builder-rust
COPY interface /src/interface
COPY frontend/Cargo.* /src/frontend/
COPY frontend/src /src/frontend/src

RUN --mount=type=cache,id=cache-rust-frontend,target=/src/frontend/target,sharing=locked \
    --mount=type=cache,id=cache-rust-interface,target=/src/interface/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    mkdir output && \
    cd /src/frontend && \
    cargo build --release && \
    cp target/release/frontend /output/


# 2. Build runtime container
# TODO switch to trixie as soon as images are available
FROM debian:trixie AS runtime

COPY --from=builder-rust /output/frontend /app/frontend/
COPY frontend/queries /app/frontend/queries
COPY frontend/references /app/frontend/references
COPY frontend/static /app/frontend/static
COPY frontend/templates /app/frontend/templates
COPY frontend/Rocket.toml /app/frontend/

WORKDIR /app/frontend

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

CMD ["/app/frontend/frontend"]
