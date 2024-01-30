FROM rust:latest

WORKDIR /usr/src/boat-core-v2

COPY config.toml ./config.toml
COPY ./target/release/boat-core-v2 ./boat-core-v2

CMD ["/usr/src/boat-core-v2/boat-core-v2"]