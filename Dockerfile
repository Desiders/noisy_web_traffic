FROM debian:buster-slim AS base
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .

FROM rust:1.63-buster AS build
WORKDIR /usr/src/app
RUN USER=root cargo init
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release
COPY ./src ./src
# https://users.rust-lang.org/t/dockerfile-with-cached-dependencies-does-not-recompile-the-main-rs-file/21577
RUN touch src/main.rs && cargo build --release

FROM base AS final
WORKDIR /app
COPY --from=build /usr/src/app/target/release/noisy_web_traffic .
ENV RUST_BACKTRACE=full
ENTRYPOINT ["/app/noisy_web_traffic"]
