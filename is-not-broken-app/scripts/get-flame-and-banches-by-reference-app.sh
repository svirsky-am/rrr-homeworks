#!/usr/bin/env bash

RUSTFLAGS="-C force-frame-pointers=yes " cargo build  \
    --manifest-path ./repos/reference-app/Cargo.toml \
    --release 

RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 4000 \
    --manifest-path ./repos/reference-app/Cargo.toml \
    --release --root --bench baseline \
    --output artifacts/generated/reference-app-flamegraph-test-integration.svg

cargo bench --bench baseline \
    --manifest-path ./repos/reference-app/Cargo.toml > artifacts/generated/reference-app-criterion-bench.txt
