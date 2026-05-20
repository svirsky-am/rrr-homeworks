

RUSTFLAGS="-C debuginfo=2 " cargo build  \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml

rust-gdb repos/origin-of-broken-app/target/debug/demo -x scripts/run-gdb-sum_even-onorigon-broken-app.gdb

