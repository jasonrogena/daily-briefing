# Stage 1: Build
FROM rust:1-slim-bookworm AS builder

# Disable HTTP/2 — required for reliable crate downloads under QEMU (arm64 emulation).
ENV CARGO_HTTP_MULTIPLEXING=false

WORKDIR /build
COPY . .
# Cache the cargo registry and build artifacts across runs to avoid re-downloading
# crates on every code change. The binary is copied out before the cache mount
# disappears so the next stage can pick it up.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build --release --bin daily-briefing \
    && cp target/release/daily-briefing /daily-briefing

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates wget \
    && ARCH=$(uname -m) \
    && wget -q "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_${ARCH}.tar.gz" -O /tmp/piper.tar.gz \
    && tar -xzf /tmp/piper.tar.gz -C /usr/local \
    && ln -s /usr/local/piper/piper /usr/local/bin/piper \
    && rm /tmp/piper.tar.gz \
    && apt-get remove -y wget \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /daily-briefing /usr/local/bin/daily-briefing

ENTRYPOINT ["/usr/local/bin/daily-briefing"]
CMD ["run", "--config", "/etc/daily-briefing/config.toml"]
