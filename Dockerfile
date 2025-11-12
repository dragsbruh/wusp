FROM rust:alpine AS chef

RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef --locked

WORKDIR /app

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest AS runtime

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/wusp /usr/bin/wusp

ENTRYPOINT ["/usr/bin/wusp"]
CMD [ "--help" ]
