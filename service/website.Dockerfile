# Builds are incremental, i.e., code changes should compile pretty fast.
# To wipe the build caches and do a fresh rebuild of everything: docker builder prune --filter type=exec.cachemount


# 1. Build Rust Application
FROM rust:trixie AS builder-rust
COPY interface /src/interface
COPY website/Cargo.* /src/website/
COPY website/src /src/website/src

RUN --mount=type=cache,id=cache-rust-website,target=/src/website/target,sharing=locked \
    --mount=type=cache,id=cache-rust-interface,target=/src/interface/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    mkdir output && \
    cd /src/website && \
    cargo build --release && \
    cp target/release/website /output/


# 2. Build runtime container
FROM debian:trixie AS runtime

COPY --from=builder-rust /output/website /app/website/
COPY website/queries /app/website/queries
COPY website/static /app/website/static
COPY website/templates /app/website/templates
COPY website/Rocket.toml /app/website/

WORKDIR /app/website

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

CMD ["/app/website/website"]
