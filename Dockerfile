FROM rust:slim AS build
WORKDIR /work
COPY . .

# Dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev curl && \
    rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Build
RUN rustup target add wasm32-unknown-unknown
RUN cargo leptos build --release


FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app

COPY --from=build /work/target/release/webtv webtv
COPY --from=build /work/target/site site

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 8080
ENTRYPOINT ["/app/webtv"]
