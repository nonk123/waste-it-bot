FROM rust:1.97-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch --locked
RUN cargo build --release
RUN rm src/main.rs

COPY src src
RUN touch src/main.rs
RUN cargo build --release

FROM debian:trixie-slim AS runner

RUN apt update \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes ca-certificates \
    && DEBIAN_FRONTEND=noninteractive apt-get dist-upgrade --yes

COPY --from=builder /app/target/release/waste-it-bot /usr/local/bin/waste-it-bot

CMD ["waste-it-bot"]
