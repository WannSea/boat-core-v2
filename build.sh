#!/bin/bash
# Note: In order to build for our Raspberry Pi you need to run this on an arm64 device!
cargo update
cargo build --release

VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')

docker build -t ghcr.io/wannsea/boat-core-v2:${VERSION} -t ghcr.io/wannsea/boat-core-v2:latest  .
docker push ghcr.io/wannsea/boat-core-v2 --all-tags
