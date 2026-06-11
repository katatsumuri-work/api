# syntax=docker/dockerfile:1

# ============================================================
# builder ステージ: musl で static link した実行バイナリを作る
# ============================================================
FROM rust:1-alpine AS builder

# musl-dev: musl ターゲットでの C リンクに必要
# curl / ca-certificates: utoipa-swagger-ui の build.rs が SwaggerUI 資材を
#   ビルド時に curl でダウンロードするため
RUN apk add --no-cache musl-dev curl ca-certificates

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets

# cargo の registry と target を BuildKit のキャッシュマウントに載せて
# 依存の再コンパイルを避ける。ビルド後、成果物をマウント外へ取り出す
# （キャッシュマウントはイメージレイヤに残らないため）。
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    && cp target/release/katatsumuri-api /app/katatsumuri-api

# ============================================================
# runtime ステージ: バイナリ 1 個だけの最小イメージ
# ============================================================
# distroless/static は shell もパッケージも持たず、攻撃面が最小。
# nonroot タグで非 root ユーザ（uid 65532）として起動する。
FROM gcr.io/distroless/static-debian12:nonroot AS runtime

COPY --from=builder /app/katatsumuri-api /usr/local/bin/katatsumuri-api

# コンテナ外から到達できるよう 0.0.0.0 で待ち受ける。
# 既存の config.rs が API_BIND_ADDR を参照するため、コード変更は不要。
ENV API_BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/katatsumuri-api"]
