#!/bin/bash
# Docker Build script for arm64 target
cargo update
cargo build --release --target aarch64-unknown-linux-gnu

VERSION=$(cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')

docker build -t ghcr.io/wannsea/boat-core-v2:${VERSION} -t ghcr.io/wannsea/boat-core-v2:latest  .
docker push ghcr.io/wannsea/boat-core-v2 --all-tags
