
module_5/reference-app
module_5/reference-app-criterion-via-black-box
module_5/origin-of-broken-app
module_5/fix-solution-of-broken-app

 rust-lldb 

 rust-gdb

# На Linux
```sh 
rust-gdb target/debug/debug-profile-practice

```

Внутри сессии GDB:

```gdb
source debug.gdb
```

```sh
rust-gdb target/debug/demo
gdb -x debug.gdb ./my_program
```

# На macOS
```
rust-lldb target/debug/demo
```

```sh


cargo check

cargo test
```
# Sampling: периодические снимки состояния
```sh 
sudo perf record -g ./target/debug/demo
```



target/debug/demo


# debug symbols
```
objdump --dwarf=info target/debug/instrumentation-demo | less 
llvm-dwarfdump --debug-line target/debug/instrumentation-demo | head -n 40 
readelf -S target/debug/instrumentation-demo | grep debug 
```


# Troubles

## git lfs track
```sh
git lfs env
git lfs track "artifacts/generated/*.csv"
git lfs track "./artifacts/**/*.csv"
git lfs ls-files
git show HEAD:artifacts/committed/reference-app-flamegraph-test-integration.svg
git show HEAD:artifacts/committed/reference-app-flamegraph-test-integration.svg
```

## add reference-app as git submodule
```sh
git submodule init
git submodule add -b module_5/reference-app git@github.com:svirsky-am/rrr-homeworks.git reference-app
git submodule update --recursive
```

## Разрешить профилирование пользовательских процессов (рекомендуется)
```
sudo sysctl -w kernel.perf_event_paranoid=1
```

## install flamegraph

```sh

cargo install rustfilt
cargo install demangle
git clone https://github.com/brendangregg/FlameGraph /tmp/FlameGraph
realpath ./FlameGraph
```




## feature `edition2024` is required 
```
error: failed to parse manifest at `/home/svirsky/repos/yandex/reference-app/Cargo.toml`

Caused by:
  feature `edition2024` is required

  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.75.0).
  Consider adding `cargo-features = ["edition2024"]` to the top of Cargo.toml (above the [package] table) to tell Cargo you are opting in to use this unstable feature.
```
### fix 
Install
```sh
rustup default nightly
```
add to `Cargo.toml`
```toml
cargo-features = ["edition2024"]
``` 

