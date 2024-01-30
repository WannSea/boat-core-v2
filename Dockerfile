FROM rust:latest

RUN apt update && \
    apt install -y protobuf-compiler

WORKDIR /usr/src/boat-core-v2
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/boat-core-v2/target \
    cargo install --path .
CMD ["boat-core-v2"]