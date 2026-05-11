
module_5/reference-app
module_5/reference-app-criterion-via-black-box
module_5/origin-of-broken-app
module_5/fix-solution-of-broken-app

 rust-lldb 

 rust-gdb

# 1. Правка UB на для sum_even
Для нагляжности можно вынесем конец диапазона в переменную `len_of_arr`  и.
Поставим на ней точку останова на 11 ой строчке `src/lib.rs` и запустим дебагер. Видно что итретор следует до `idx=4`, т.к. values.len()=4, но индексация массива начинается с `0` и по хоршему надо выходить  при  `idx=3` чтобы не выйти за границы массива. 
![Состояние переменных перед вызоввом *values.get_unchecked(idx)](artifacts/committed/1.UB_sum_even.png)

## Воспроизведение:
Запуск дебаггера оригинальном broken-app:
```sh 
RUSTFLAGS="-C debuginfo=2 " cargo build  \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml
rust-gdb repos/origin-of-broken-app/target/debug/demo -x scripts/run-gdb-sum_even-onorigon-broken-app.gdb 
```

Получаем:
```
(gdb) q#16 0x000055555556ea44 in broken_app::sum_even (values=&[i64](size=4) = {...}) at src/lib.rs:11
11                  let v = *values.get_unchecked(idx);
(gdb)  info locals 
idx = 4
iter = core::ops::range::RangeInclusive<usize> {start: 4, end: 4, exhausted: true}
acc = 6
```
## Быстрый фикс

Решение : вычесть из конца диапазона `1`:
```rs
pub fn sum_even(values: &[i64]) -> i64 {
    let mut acc = 0;
    unsafe {
        for idx in 0..=values.len() -1  { // теперь мы обращаеся к массиву от 0 до 3.
            let v = *values.get_unchecked(idx);
            if v % 2 == 0 {
                acc += v;
            }
        }
    }
    acc
}
```
## Альтернативное решение
Самое быстрое решение (похоже за счет оптимизаций компилдятора) ~546ns против   sum_even: ~164.27µs в reference app
```rs
let mut acc: i64 = 0;
for &v in values {
    if (v & 1) == 0 { acc += v; }
}
acc
```
Получение пруфов:
```sh
scripts/get-flame-and-banches-by-broken-app.sh
scripts/get-flame-and-banches-by-reference-app.sh
```


# 2. Анализ fn normalize

Как таковой "небрежной нормализация строки" не замечено, т.к. она просто не полная, если нужно еще зачищать '\n' и `\t`. Двойные пробелы в текущей реализации удаляютс корректно .

В файл `src/lib.rs` добавлена оригинальная функция `normalyze` из `reference_app` под названием `normalize_by_reference_app` ускореенная функция `normalize_faster_new_alt`. Для функций добвлены bench'и для сравнения скорости работы.
Запустим 
```sh 
 cargo bench  --bench criterion
 ```
В выводе наблюдаем уеличение скорости в 3-5 раз по сравнения с reference-app.
```sh
...
normalize_by_reference_app
                        time:   [2.7244 µs 2.7582 µs 2.8010 µs]
   ...
normalize_faster_new_alt
                        time:   [774.71 ns 787.63 ns 805.05 ns]
```
## За счет чего увеличена скорость?
|Примененные оптимизации |   Что даёт|
|---|---|
|Работа с &[u8] вместо chars() | Пропускаем декодер UTF-8. ASCII-символы занимают 1 байт, проверка идёт напрямую.|
|Один проход (O(n)) | Не создаём промежуточные итераторы, аллокаторы или временные строки c последующим сбором вектора|
|'b|32'|`b|32' вместоto '_lowercase()`|
|Vec::with_capacity(src.len()) | Выделение памяти происходит один раз. Итераторные цепочки часто делают push без резерва, вызывая реаллокации.|
|LLVM Auto-Vectorization | В --release компилятор превращает этот цикл в SIMD-инструкции (AVX2/SSE4.1), обрабатывая по 16–32 байта за такт.|


Пример приведения символов ASCII к нижнему регистру с помощью `b|32`:
```rs
    let char_upper: u8 = b'A'; // 65 (0100 0001)
    let char_lower = char_upper | 32; // 65 | 32 = 97 (0110 0001)
```
Заглавные ASCII буквы ('A'-'Z') имеют коды 65-90.Строчные ASCII буквы ('a'-'z') имеют коды 97-122. Разница между ними — ровно \(32\) (\(2^{5}\)). 6-й бит (считая с 0) у заглавных букв равен 0, а у строчных — 1.


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
git submodule add -b module_5/reference-app git@github.com:svirsky-am/rrr-homeworks.git repos/reference-app
git submodule add -b module_5/origin-of-broken-app git@github.com:svirsky-am/rrr-homeworks.git repos/origin-of-broken-app
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

