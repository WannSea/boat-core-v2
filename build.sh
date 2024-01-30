#!/bin/bash
cargo update
cargo build --release
docker build --platform linux/arm64 -t ghcr.io/wannsea/boat-core-v2:latest .
docker push ghcr.io/wannsea/boat-core-v2:latest
