#!/usr/bin/env bash

RUSTFLAGS="-C force-frame-pointers=yes " cargo build --release

RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 400 \
    --release --root --bench baseline \
    --output artifacts/generated/broken-app-flamegraph-test-integration.svg

cargo  bench  --bench baseline  > artifacts/generated/broken-app-criterion-bench.txt

# cargo bench -- --save-baseline v1
# Сохранить результаты как baseline для будущего сравнения
# cargo bench -- --baseline v1
# Сравнить текущий прогон с сохранённым baseline
# cargo bench -- --sample-size 200
# Увеличить точность (по умолчанию 100)