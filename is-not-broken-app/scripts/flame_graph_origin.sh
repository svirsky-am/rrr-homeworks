#!/usr/bin/env bash
set -euo pipefail 


RUSTFLAGS="-C debuginfo=2" cargo build --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --bin demo_cicle_1000 --lib

RUSTFLAGS="-C debuginfo=2" cargo build --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --bin demo_cicle_1000 --lib
sudo -E env "PATH=$PATH" cargo flamegraph -F 999999 --bin demo_cicle_1000


 cargo build --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --bin demo_cicle_1000 --lib
cargo flamegraph -F 400 --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --root --test integration --output artifacts/reference-app-tests.svg

cargo flamegraph -F 400 --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml --release --root --bench baseline --output artifacts/reference-app-bench-baseline.svg


RUSTFLAGS="-C force-frame-pointers=yes " cargo build  \
    --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml \
    --release 

RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 4000 \
    --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml \
    --release --root --bench baseline \
    --output artifacts/reference-app-bench-baseline.svg

cargo bench --bench baseline \
    --manifest-path /home/svirsky/repos/yandex/reference-app/Cargo.toml 



RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
--title 
--root
--output
--release


git submodule add -b module_5/reference-app git@github.com:svirsky-am/rrr-homeworks.git reference-app

git submodule add <repository_url> <path_to_directory>