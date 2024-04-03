FROM debian:bookworm-slim

WORKDIR /usr/src/boat-core-v2

COPY config.toml ./config.toml
COPY ./target/aarch64-unknown-linux-gnu/release/boat-core-v2 ./boat-core-v2

CMD ["/usr/src/boat-core-v2/boat-core-v2"]
