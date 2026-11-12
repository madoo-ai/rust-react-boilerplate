# syntax=docker/dockerfile:1

FROM node:24-bookworm-slim AS web
WORKDIR /app/web
ARG VITE_APP_NAME="Rust React Boilerplate"
ENV VITE_APP_NAME=$VITE_APP_NAME
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

FROM rust:1.96-bookworm AS api
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY api ./api
RUN cargo build --release -p rust-react-boilerplate-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app --home-dir /app app

WORKDIR /app
COPY --from=api /app/target/release/rust-react-boilerplate-api /app/api
COPY --from=web /app/web/dist /app/web/dist

ENV APP_URL=http://localhost:3000 \
    APP_NAME="Rust React Boilerplate" \
    APP_LOCALE=id \
    APP_TIME_ZONE=Asia/Jakarta \
    PORT=3000 \
    WEB_DIST=/app/web/dist \
    COOKIE_SECURE=false \
    RUST_LOG=info,api=debug,rust_react_boilerplate_api=debug

RUN chown -R app:app /app
USER app
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/api/health || exit 1

ENTRYPOINT ["/app/api"]
CMD ["serve"]
