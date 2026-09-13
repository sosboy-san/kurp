# --- ビルドステージ ---
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# ソースコード一式をコピー
COPY . .

# nasm を追加（AVIFエンコーダ rav1e のビルドに必須）
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    nasm \
    && rm -rf /var/lib/apt/lists/* \
    && cargo build --release

# --- 実行ステージ（超軽量・GPU不要） ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# ビルド成果物のみを配置
COPY --from=builder /app/target/release/kurp /usr/local/bin/kurp

ENV KURP_CONF_DIR="/config"
ENV RUST_LOG=info
EXPOSE 3030

RUN mkdir -p /config /cache

ENTRYPOINT ["kurp"]
