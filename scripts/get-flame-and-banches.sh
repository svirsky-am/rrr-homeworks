#!/usr/bin/env bash

# RUSTFLAGS="-C debuginfo=2" cargo build --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --bin demo_cicle_1000 --lib

# RUSTFLAGS="-C debuginfo=2" cargo build --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --bin demo_cicle_1000 --lib
# sudo -E env "PATH=$PATH" cargo flamegraph -F 999999 --bin demo_cicle_1000


#  cargo build --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --bin demo_cicle_1000 --lib
# cargo flamegraph -F 400 --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --root --test integration --output artifacts/reference-app-tests.svg

# cargo flamegraph -F 400 --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --root --bench baseline --output artifacts/reference-app-bench-baseline.svg


RUSTFLAGS="-C force-frame-pointers=yes " cargo build  \
    --manifest-path ./reference-app/Cargo.toml \
    --release 

# RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 4000 \
#     --manifest-path ./reference-app/Cargo.toml \
#     --release --test integration --root \
#     --output artifacts/generated/reference-app-flamegraph-test-integration.svg

RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 4000 \
    --manifest-path ./reference-app/Cargo.toml \
    --release --root --bench baseline \
    --output artifacts/generated/reference-app-flamegraph-test-integration.svg



cargo bench --bench baseline \
    --manifest-path ./reference-app/Cargo.toml > artifacts/generated/reference-app-criterion-bench.txt



# RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
# --title 
# --root
# --output
# --release

git submodule init reference-app
