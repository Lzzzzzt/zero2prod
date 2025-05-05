FROM lukemathwalker/cargo-chef:latest-rust-1.86-alpine AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev clang ca-certificates pkgconfig openssl-dev

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod

FROM alpine:latest AS runtime
WORKDIR /app
RUN apk add --no-cache ca-certificates openssl
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY config/ config/
ENV APP_ENVIRONMENT="prod"
ENTRYPOINT [ "./zero2prod" ]
