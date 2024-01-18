FROM rust:latest

RUN apt update && \
    apt install -y protobuf-compiler

WORKDIR /usr/src/boat-core-v2
COPY . .
RUN cargo install --path .
CMD ["boat-core-v2"]