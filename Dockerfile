# Stage 1: Build judge binary
FROM rust:latest AS builder

WORKDIR /usr/src/furaoj-judgeserver-rust

RUN apt-get update && apt-get install -y --no-install-recommends \
    libseccomp-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Stage 2: Sandboxed runner with compilers
FROM debian:trixie-slim AS runner

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libseccomp2 \
    gcc \
    g++ \
    python3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /var/furaoj/problems /tmp/sandbox

COPY --from=builder /usr/src/furaoj-judgeserver-rust/target/release/furaoj-judgeserver-rust /app/furaoj-judgeserver-rust

ENTRYPOINT ["/app/furaoj-judgeserver-rust"]
CMD ["/app/judge_config.json"]
