#!/usr/bin/env bash

RUSTFLAGS="-C debuginfo=2" cargo run -- \
    --test integration 