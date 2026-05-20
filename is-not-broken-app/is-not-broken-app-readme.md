# Оглавление

- [Оглавление](#оглавление)
- [0. Допущения](#0-допущения)
- [Шаг 1. Ознакомление](#шаг-1-ознакомление)
  - [1.1 Клонируйте оба проекта (broken-app, reference-app)](#11-клонируйте-оба-проекта-broken-app-reference-app)
  - [1.2 Фиксация падающих тестов в `origin-of-broken-app`](#12-фиксация-падающих-тестов-в-origin-of-broken-app)
  - [1.3 Воспроизведение ожидаемого результата](#13-воспроизведение-ожидаемого-результата)
- [Шаг 2. Поиск и исправление багов](#шаг-2-поиск-и-исправление-багов)
  - [2.1 Правка UB для `sum_even`](#21-правка-ub-для-sum_even)
    - [2.1.1 Воспроизведение](#211-воспроизведение)
    - [2.1.2 Быстрый фикс](#212-быстрый-фикс)
  - [2.2 Правка фильтра положительных значений для `averages_only_positive`](#22-правка-фильтра-положительных-значений-для-averages_only_positive)
  - [2.3 Запуск `cargo +nightly miri test` для поиска UB](#23-запуск-cargo-nightly-miri-test-для-поиска-ub)
  - [2.4 Фикс `leak_buffer`](#24-фикс-leak_buffer)
  - [2.5 Запуск с Valgrind](#25-запуск-с-valgrind)
    - [2.5.1 ASan](#251-asan)
    - [2.5.2 TSan](#252-tsan)
  - [2.6 Анализ срабатываний TSan](#26-анализ-срабатываний-tsan)
- [Шаг 3. Подтверждение корректности](#шаг-3-подтверждение-корректности)
  - [3.1 Добавление регрессионных тестов](#31-добавление-регрессионных-тестов)
    - [3.1.1 Для регрессионных юнит-тестов `leak_buffer`](#311-для-регрессионных-юнит-тестов-leak_buffer)
    - [3.1.2 Фикс теста](#312-фикс-теста)
  - [3.2 Контрольная проверка фиксов](#32-контрольная-проверка-фиксов)
  - [3.3 Дополнение имеющихся юнит-тестов](#33-дополнение-имеющихся-юнит-тестов)
    - [3.3.1 Тест `normalize_simple`](#331-тест-normalize_simple)
    - [3.3.2 Тесты для функций `race_increment`, `read_after_sleep` и `reset_counter`](#332-тесты-для-функций-race_increment-read_after_sleep-и-reset_counter)
    - [3.3.3 Фиксы для функций `race_increment`, `read_after_sleep`, `reset_counter` и приложения `demo_for_threats`](#333-фиксы-для-функций-race_increment-read_after_sleep-reset_counter-и-приложения-demo_for_threats)
- [Шаг 4. Поиск узких мест](#шаг-4-поиск-узких-мест)
  - [4.1 Построение flamegraph для demo](#41-построение-flamegraph-для-demo)
- [Шаг 5. Бенчмарки до оптимизации](#шаг-5-бенчмарки-до-оптимизации)
  - [5.1 Настройка бенчмарков для broken\_app и reference\_app](#51-настройка-бенчмарков-для-broken_app-и-reference_app)
    - [5.1.1 Правка вызова criterion в `broken_app`](#511-правка-вызова-criterion-в-broken_app)
- [Шаг 6. Оптимизация](#шаг-6-оптимизация)
  - [6.1 Микрооптимизация `sum_even`](#61-микрооптимизация-sum_even)
  - [6.2 Оптимизация `average_positive`](#62-оптимизация-average_positive)
  - [6.3 Оптимизация `normalize`](#63-оптимизация-normalize)
    - [За счёт чего увеличена скорость?](#за-счёт-чего-увеличена-скорость)
  - [6.4 Оптимизация `slow_dedup` и `slow_fib`](#64-оптимизация-slow_dedup-и-slow_fib)
- [Шаг 7. Проверка «после»](#шаг-7-проверка-после)
  - [7.1 Резюме](#71-резюме)
- [Troubleshooting](#troubleshooting)
  - [Инструментированная сборка с Miri](#инструментированная-сборка-с-miri)
  - [Инструментированная сборка с Valgrind](#инструментированная-сборка-с-valgrind)
  - [Debug symbols](#debug-symbols)
  - [Git LFS track](#git-lfs-track)
  - [Добавление reference-app как git submodule](#добавление-reference-app-как-git-submodule)
  - [Разрешить профилирование пользовательских процессов (рекомендуется)](#разрешить-профилирование-пользовательских-процессов-рекомендуется)
  - [Установка flamegraph](#установка-flamegraph)
  - [Ошибка `feature edition2024 is required`](#ошибка-feature-edition2024-is-required)
    - [Fix](#fix)

---

# 0. Допущения

- Для `race_increment`, `read_after_sleep` нет юнит-тестов, а `reset_counter` не описан в `reference_app`. Работоспособность воспроизведена на своё усмотрение;
- В тесте `normalize` нет кейса на проверку `\n`, `\t`. Поэтому правка выполнена чуть позже;
- В предоставленных `broken_app` и `reference_app` не работают тесты criterion. Работоспособность воспроизведена на своё усмотрение;
- Наглядно оформить сбор логов только за счёт коммитов и bash-скриптов оказалось тяжело. Был применён git submodules с отображением коммитов на каждом шаге ДЗ;
- Для честности измерений старые реализации с «горячими фиксами» (hot fix), реализации `reference-app` и новые оптимизации оформлены в одной библиотеке. Замеры будут вызываться в рамках одной сессии criterion.

# Шаг 1. Ознакомление

## 1.1 Клонируйте оба проекта (broken-app, reference-app)

Сделаем доступными исходники для получения логов и отчётов с помощью git submodules:

```sh
git submodule update --recursive
```

В папку `repos` будут выгружены экземпляры `broken_app` в разных состояниях:
- `reference-app` — ветка `module_5/reference-app` с ожидаемым поведением тестов `broken_app`;
- `origin-of-broken-app` — ветка `module_5/origin-of-broken-app` с оригинальными исходниками `broken-app`;
- `origin-of-broken-app-fix-unit` — ветка `module_5/origin-of-broken-app`, оригинальные исходники `broken-app` с правкой unit-тестов;
- `origin-of-broken-app-with-hot-fix` — ветка `module_5/origin-of-broken-app-with-hot-fix` с простыми фиксами тестов без оптимизаций;
- `broken-app-2.4-fix-leak-buffer-after-miri` — ветка `module_5/roken-app-2.4-fix-leak-buffer-after-miri` с фиксом утечки `fn leak_buffer`;
- `broken-app-3.1.1-add-unit-tests-for-leak-buff` — расширение unit-тестов;
- `broken-app-3.3-extra-tests-for-normalyze-and-threat` — расширение unit-тестов;
- `broken-app-3.3.3-fix-tsan-for-demo-for-threats` — ветка с фиксом функций из `src/concurrency.rs`;
- `broken-app-5.1-fixup-criterion-for-broken-app` — ветка с правкой бенчмарка criterion;
- `broken-app-6-optimized` — ветка с применёнными оптимизациями;
- `broken-app-7-final` — ветка для контрольных проверок;

<!-- - module_5/fix-solution-of-broken-appe -->
<!-- - module_5/reference-app-criterion-via-black-box -->

## 1.2 Фиксация падающих тестов в `origin-of-broken-app`

Проверим запуск тестов:

```sh
cargo check \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml
cargo test \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml
```

Получаем падение для `sums_even_numbers`:

```sh
cargo test \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml sums_even_numbers
```

Получаем:
> thread 'sums_even_numbers' (2137351) panicked at src/lib.rs:11:29:
> unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice

И для `averages_only_positive`:

```sh
cargo test  \
        --manifest-path ./repos/origin-of-broken-app/Cargo.toml averages_only_positive
```

С ошибкой:
> thread 'averages_only_positive' (2138421) panicked at tests/integration.rs:36:5:
> assertion failed: (broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON
> note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

## 1.3 Воспроизведение ожидаемого результата

Все assert'ы от тестов прошли без паник:

```sh
cargo test  \
        --manifest-path ./repos/reference-app/Cargo.toml
```

В stdout видим результат:
> test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Шаг 2. Поиск и исправление багов

## 2.1 Правка UB для `sum_even`

Паника после запуска теста отсылает к месту падения в исходниках `src/lib.rs:11`. Поставим точку останова на 11-й строчке `src/lib.rs` и запустим отладчик. Видно, что итератор доходит до `idx=4`, т.к. `values.len()=4`, но индексация массива начинается с `0`, а последний элемент имеет индекс `3`, и, по-хорошему, надо выходить при `idx=3`, чтобы не выйти за границы массива.

![Состояние переменных перед вызовом *values.get_unchecked(idx)](artifacts/committed/1.UB_sum_even.png)

### 2.1.1 Воспроизведение

Запуск отладчика на оригинальном broken-app:

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

### 2.1.2 Быстрый фикс

Решение: вычесть из конца диапазона `1`:

```rs
pub fn sum_even(values: &[i64]) -> i64 {
    let mut acc = 0;
    unsafe {
        for idx in 0..=values.len() -1  { // теперь мы обращаемся к массиву от 0 до 3.
            let v = *values.get_unchecked(idx);
            if v % 2 == 0 {
                acc += v;
            }
        }
    }
    acc
}
```

Применим фикс на репозитории `repos/origin-of-broken-app-with-hot-fix` и запустим тест:

```sh
cargo test \
    --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml \
    sums_even_numbers
```

Тест пройден.

## 2.2 Правка фильтра положительных значений для `averages_only_positive`

Соберём тест:

```sh
export TEST_EXEC_PATH=$(cargo test --no-run --manifest-path ./repos/origin-of-broken-app/Cargo.toml \
 averages_only_positive --message-format=json 2>/dev/null | jq -r 'select(.reason == "compiler-artifact" ) | .executable ' | grep  integration)
# проверяем то что бинарник нашелся
echo "Путь к бинарнику теста: $TEST_EXEC_PATH" 
# Путь к бинарнику теста: /home/svirsky/repos/yandex/broken-app/repos/origin-of-broken-app/target/debug/deps/integration-d7b56cc9cfa7dfa1
```

Проверяем запуск теста:

```sh
$TEST_EXEC_PATH # 
```

Наблюдаем срабатывание assert'ов:
> assertion failed: (broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON

Таким образом, нас интересует результат, возвращаемый `broken_app::average_positive(&nums)`. Запускаем отладчик:

```sh
rust-gdb $TEST_EXEC_PATH
```

Делаем точку останова на возврате функции:

```sh #(gdb)
break src/lib.rs:52
#Breakpoint 1 at 0x7a670: file src/lib.rs, line 52.
run
# Thread 2 "averages_only_p" hit Breakpoint 1, broken_app::average_positive (values=&[i64](size=3) = {...}) at src/lib.rs:52
info args
#values = &[i64](size=3) = {-5, 5, 15}
info locals
#sum = 15
```

Делаем вывод о том, что элементы массива, скорее всего, сложились как `(-5)+5+15`. Применим фильтр при сложении элементов:

```rs
pub fn average_positive(values: &[i64]) -> f64 {
    let sum: i64 = values.iter().filter(|&&x| x > 0).sum();
    if values.is_empty() {
        return 0.0;
    }
    sum as f64 / values.iter().filter(|&&x| x > 0).count() as f64
}
```

```sh
cargo test --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml \
 averages_only_positive
```

Контрольная проверка запуска тестов:

```sh
cargo  test --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml
```

## 2.3 Запуск `cargo +nightly miri test` для поиска UB

```sh
MIRIFLAGS=-Zmiri-backtrace=full cargo  miri test --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml
```

В логах (`artifacts/committed/2_3_miri_after_hot_fix.log`) в стеке видим:
>         7: broken_app::leak_buffer
>                at src/lib.rs:21:17: 21:31

Похоже, мы вызываем `Box::into_raw`, но не освобождаем память через `Box::from_raw`.

## 2.4 Фикс `leak_buffer`

Применим правку утечки на базе ветки `module_5/origin-of-broken-app-with-hot-fix`, скопировав решение в ветку `module_5/broken-app-2.4-fix-leak-buffer-after-miri`.

<!-- git submodule add -b module_5/origin-of-broken-app-with-hot-fix git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-2.4-fix-leak-buffer-after-miri -->

Правка будет заключаться в освобождении бокса:

```rs
pub fn leak_buffer(input: &[u8]) -> usize {
    let boxed = input.to_vec().into_boxed_slice();
    let len = input.len();
    let raw = Box::into_raw(boxed) as *mut u8;

    let mut count = 0;
    unsafe {
        for i in 0..len {
            if *raw.add(i) != 0_u8 {
                count += 1;
            }
        }
        // освобождаем память
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(raw, len)) };
    }
    count
}
```

`Box::from_raw` ожидает `*mut [u8]`, поэтому нужно восстановить слайс правильной длины через `slice::from_raw_parts_mut`.

Проверяем фикс:

```sh
cargo  miri test --manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```

Тесты пройдены без ошибок (лог `artifacts/committed/2_4_fix_leak_buffer_after_miri.log`).

## 2.5 Запуск с Valgrind

### 2.5.1 ASan

Проверим текущее состояние:

```sh
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```

По логу `artifacts/committed/mod5_2_5_1_valgrind_asan_first_one.log` ошибок не обнаружено.

### 2.5.2 TSan

```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer --leak-check=full --show-leak-kinds=all" cargo +nightly test \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```

По логу `artifacts/committed/mod5_2_5_2_valgrind_tsan_first_one.log` наблюдаем срабатывания TSan:

```
/usr/bin/addr2line: DWARF error: invalid or unhandled FORM value: 0x23
==================
WARNING: ThreadSanitizer: data race (pid=2359557)
  Write of size 8 at 0x729400000148 by thread T2:
    #0 memcpy ??:? (integration-a4fdbff301df0fb7+0x6c82e) (BuildId: 63a719d20515dfa93006ddc55268ce93f2fe41ec)
    #1 <std::sync::mpmc::Sender<test::event::CompletedTest>>::send test.905ab08d71382a9a-cgu.0:? (integration-a4fdbff301df0fb7+0x115892) (BuildId: 63a719d20515dfa93006ddc55268ce93f2fe41ec)

  Previous write of size 8 at 0x729400000148 by thread T1:
    #0 calloc ??:? (integration-a4fdbff301df0fb7+0x6f7c7) (BuildId: 63a719d20515dfa93006ddc55268ce93f2fe41ec)
    #1 <std::sync::mpmc::Sender<test::event::CompletedTest>>::send test.905ab08d71382a9a-cgu.0:? (integration-a4fdbff301df0fb7+0x114f20) (BuildId: 63a719d20515dfa93006ddc55268ce93f2fe41ec)

  Location is heap block of size 9680 at 0x729400000000 allocated by thread T1:
    #0 calloc ??:? (integration-a4fdbff301df0fb7+0x6f7c7) (BuildId: 63a719d20515dfa93006ddc55268ce93f2fe41ec)
    #1 <std::sync::mpmc::Sender<test::event::CompletedTest>>::send test.905ab08d71382a9a-cgu.0:? (integration-a4fdbff301df0fb7+0x114f20) (BuildId: 63a719d20515dfa93006ddc55268ce93f2fe41ec)

  Thread T2 'counts_non_zero' (tid=2359560, running) created by main thread at:
    #0 pthread_create ??:? (in
```

## 2.6 Анализ срабатываний TSan

Мы наблюдаем проблему с двумя потоками двух разных сессий. Починка тестового фреймворка выходит за рамки ДЗ, поэтому вместо фикса попробуем убедиться, что проблема не в `lib.rs`. Выполним аналогичный запуск для `demo.rs`:

```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly run --bin demo \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```

В логе `artifacts/committed/mod5_2_6_1_prove_leak_is_not_by_demo.log` срабатываний TSan не обнаружено. Проверим через benches:

```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly bench --bench baseline \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```

В логе `artifacts/committed/mod5_2_6_1_prove_leak_is_not_by_lib_rs___via_benches.log` срабатываний TSan не обнаружено.

> **Вывод:** Срабатывание TSan произошло в многопоточном тестовом фреймворке.

# Шаг 3. Подтверждение корректности

## 3.1 Добавление регрессионных тестов

Правки функций `sum_even` и `average_positive` покрываются имеющимися юнит-тестами.

### 3.1.1 Для регрессионных юнит-тестов `leak_buffer`

На базе `broken-app-2.4-fix-leak-buffer-after-miri` создадим ветку `broken-app-3.1.1-add-unit-tests-for-leak-buff`

<!-- git submodule add -b module_5/broken-app-2.4-fix-leak-buffer-after-miri git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-3.1.1-add-unit-tests-for-leak-buff -->

И добавим тесты:

```rs
#[test]
fn test_leak_buffer_zero_vs_nonzero_distinction() {
    assert_eq!(leak_buffer(&[0, 0, 0]), 0);
    assert_eq!(leak_buffer(&[1, 1, 1]), 3);
    assert_eq!(leak_buffer(&[0, 1, 0, 1, 0]), 2);
    assert_eq!(leak_buffer(&[255, 0, 128, 0, 1]), 3);
    let full = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    // Берём срез, начинающийся с нечётного индекса
    assert_eq!(leak_buffer(&full[1..6]), 5); // [1,2,3,4,5] → все ненулевые
    //Большие данные (проверка производительности и переполнений)
    let mut input = vec![0u8; 10_000];
    // Каждое 10-е значение ненулевое
    for i in (0..input.len()).step_by(10) {
        input[i] = 42;
    }
    assert_eq!(leak_buffer(&input), 1_000);
}

/// Все нули → результат 0
#[test]
fn test_leak_buffer_all_zeros() {
    let input = [0u8; 100];
    assert_eq!(leak_buffer(&input), 0);
}

/// Все ненулевые → результат = длина
#[test]
fn test_leak_buffer_all_non_zero() {
    let input = [1u8; 50];
    assert_eq!(leak_buffer(&input), 50);
}
```

Запускаем тесты:

```sh
cargo test \
	--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml
```

В логе `artifacts/committed/broken-app-3.1.1-add-unit-tests-for-leak-buff.log` срабатываний TSan не обнаружено.

### 3.1.2 Фикс теста

## 3.2 Контрольная проверка фиксов

Запустим имеющиеся тесты на ветке `broken-app-3.1.1-add-unit-tests-for-leak-buff`:

```sh
# miri
cargo  miri test --manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml
# asan
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml
# tsan
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml
```

Результат в `artifacts/committed/mod5_3_1_2_check_after_hot_fix.log`.

## 3.3 Дополнение имеющихся юнит-тестов

Анализируя тесты, можно заметить, что:
- Тест `normalize_simple` для функции `normalize` не покрывает кейсов, когда тестовая строка содержит в себе табуляции `\t` и переносы строк `\n`;
- Нет тестов для `race_increment`;
- Нет тестов для `read_after_sleep`;
- Нет тестов для `reset_counter`.

Сделаем правки на ветке `module_5/broken-app-3.3-extra-tests-for-normalyze-and-threat`, созданной на базе ветки `broken-app-3.1.1-add-unit-tests-for-leak-buff`.

<!-- git submodule add -b module_5/broken-app-3.1.1-add-unit-tests-for-leak-buff git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-3.3-extra-tests-for-normalyze-and-threat -->

### 3.3.1 Тест `normalize_simple`

Для проверки `normalize` в соответствии с описанием к функции нужно добавить проверяемые символы в проверочную строку:

```rs
#[test]
fn normalize_simple() {
    assert_eq!(normalize(" H  e\n\nllo                       Wo\t\t\t\t\t\t\t\t\t\t\trld "), "helloworld");
}
```

И сразу применим фикс к функции:

```rs
pub fn normalize(input: &str) -> String {
    input.replace(' ', "").replace('\n', "").replace('\t', "").to_lowercase()
}
```

Проверка:

```sh 
cargo test \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml -- normalize_simple
```

### 3.3.2 Тесты для функций `race_increment`, `read_after_sleep` и `reset_counter`

Портируем имеющийся тест `race_increment_is_correct` из reference_app и запустим его:

```sh 
cargo test \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml -- race_increment_is_correct
```

И не увидим гонки:
> running 1 test
> test race_increment_is_correct ... ok

Для запуска TSan оформим тестовое приложение `demo_for_threats`, использующее эти функции, т.к. тестовый фреймворк имеет свои ошибки при запуске TSan.

```rs
use is_not_broken_app::{concurrency};

fn main() {

    let total = concurrency::race_increment(1_000, 4);
    println!("total: {:?}", &total);
    concurrency::read_after_sleep();
    concurrency::reset_counter();
    println!("total: {:?}", &total);
    
}
```

Запустим несколько раз:

```sh
cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml 
```

Каждый раз получаем рандомные значения:
> total: 3131
> total: 3131

> total: 3856
> total: 3856

Запустим TSan на этом приложении:

```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer  -Awarnings" \
    cargo +nightly run --bin demo_for_threats \
    --target x86_64-unknown-linux-gnu \
    --manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml 
```

Получаем срабатывание:

```
WARNING: ThreadSanitizer: data race (pid=3681059)
  Read of size 8 at 0x5555565cf750 by thread T2:
    #0 broken_app::concurrency::race_increment::{closure#0} 
```

Подробности в логе `artifacts/committed/broken-app-3.3-run-bin-demo_for_threats-via-tsan.log`. Коммитимся и фиксим.

### 3.3.3 Фиксы для функций `race_increment`, `read_after_sleep`, `reset_counter` и приложения `demo_for_threats`

Фикс сделаем на базе ветки `module_5/broken-app-3.3-extra-tests-for-normalyze-and-threat`.

<!--
git submodule add -b module_5/broken-app-3.3-extra-tests-for-normalyze-and-threat git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats
pushd repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats
git checkout -b module_5/broken-app-3.3.3-fix-tsan-for-demo-for-threats
git push --set-upstream origin module_5/broken-app-3.3.3-fix-tsan-for-demo-for-threats
popd
 -->

Применим атомарные функции:

```rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

// Заменяем unsafe static mut на атомарную переменную
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Безопасный инкремент через несколько потоков.
/// Использует атомарные операции — нет data race.
pub fn race_increment(iterations: usize, threads: usize) -> u64 {
    // Сбрасываем счётчик перед началом (атомарно)
    COUNTER.store(0, Ordering::Relaxed);
    
    let mut handles = Vec::new();
    for _ in 0..threads {
        handles.push(thread::spawn(move || {
            for _ in 0..iterations {
                // Атомарное увеличение на 1
                // Relaxed достаточно для счётчика, если не нужна синхронизация с другими данными
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    // Атомарное чтение финального значения
    COUNTER.load(Ordering::Relaxed)
}

/// Чтение счётчика после небольшой задержки.
/// Теперь безопасно: атомарная загрузка.
pub fn read_after_sleep() -> u64 {
    thread::sleep(Duration::from_millis(10));
    COUNTER.load(Ordering::Relaxed)
}

/// Сброс счётчика — теперь атомарный.
pub fn reset_counter() {
    COUNTER.store(0, Ordering::Relaxed);
}
```

Немного модифицировав приложение, запустим без инструментации:

```sh
cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml 
```

Всё работает, как заявлено в названиях функций:

```
Запуск race_increment(1_000, 4)...
После инкремента: total = 4000
После sleep: counter = 4000
После reset: counter = 0
Все проверки пройдены!
```

Теперь запустим инструментальную сборку TSan:

```rs
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer  -Awarnings" cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml 
```

Теперь ошибок TSan нет. Лог: `artifacts/committed/broken-app-3.3.3-fix-tsan-for-demo-for-threats.log`.

# Шаг 4. Поиск узких мест

## 4.1 Построение flamegraph для demo

Для начала попробуем построить flamegraph для приложения `demo`:

```sh
RUSTFLAGS="-C force-frame-pointers=yes " cargo build --release \
    --manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml
RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 999 \
    --release --root --bin demo \
    --manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml  \
    --output artifacts/generated/4.1_flamegraph_bin_demo.svg
```

На графике мы не видим интересующих нас функций, т.к. программа заканчивается очень быстро.

![HOT_segments](artifacts/committed/4.1_flamegraph_bin_demo.svg)

Если мы добавим цикл, то, скорее всего, увидим системные вызовы, связанные с выводом данных в stdout. Поэтому попробуем построить flamegraph для benches:

```sh
RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 400 \
    --release --root --bench baseline \
    --manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml  \
    --output artifacts/generated/4.1_flamegraph_benches.svg
```

Тут мы видим два сегмента, которые можно немного ускорить. Выполним это в разделе с оптимизациями.

![HOT_segments](artifacts/committed/Explane_4.1._benches.png)

# Шаг 5. Бенчмарки до оптимизации

## 5.1 Настройка бенчмарков для broken_app и reference_app

### 5.1.1 Правка вызова criterion в `broken_app`

На оригинальном broken-app с hot-фиксами не работает бенчмарк criterion. На базе ветки `module_5/broken-app-3.3.3-fix-tsan-for-demo-for-threats` с хотфиксами оригинального `broken-app` подготовим ветку для сбора бенчмарков.

<!--
git submodule add -b module_5/broken-app-3.3.3-fix-tsan-for-demo-for-threats git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-5.1-fixup-criterion-for-broken-app
pushd repos/broken-app-5.1-fixup-criterion-for-broken-app
git checkout -b module_5/broken-app-5.1-fixup-criterion-for-broken-app
git push --set-upstream origin module_5/broken-app-5.1-fixup-criterion-for-broken-app
popd
 -->

В `Cargo.toml` нужно добавить:

```toml
[[bench]]
name = "criterion"  # Должен совпадать с именем файла в benches/
harness = false         #  Отключает libtest
```

Запустим бенчмарк:

```sh
cargo bench  --bench criterion \
    --manifest-path ./repos/broken-app-5.1-fixup-criterion-for-broken-app/Cargo.toml
```

Наблюдаем результаты в формате:

```
...
sum_even_broken         time:   [146.57 µs 150.48 µs 154.83 µs]
                        change: [-16.346% -12.940% -9.5176%] (p = 0.00 < 0.05)
                        Performance has improved.
...
```

Подробности в логе `artifacts/committed/broken-app-5.1.1-fixup-criterion.log`. Для честности измерений старые реализации с «горячими фиксами» (hot fix), реализации `reference-app` и новые оптимизации оформлены в одной библиотеке. Замеры будут вызываться в рамках одной сессии criterion.

# Шаг 6. Оптимизация

Оптимизации будут выполнены на ветке `module_5/broken-app-6-optimized` на базе ветки `module_5/broken-app-5.1-fixup-criterion-for-broken-app`.

<!--
git submodule add -b module_5/broken-app-5.1-fixup-criterion-for-broken-app git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-6-optimized
pushd repos/broken-app-6-optimized
git checkout -b module_5/broken-app-6-optimized
git push --set-upstream origin module_5/broken-app-6-optimized
popd
 -->

## 6.1 Микрооптимизация `sum_even`

```sh
RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -F 400 \
    --release --root --bench baseline_sum_even  \
    --manifest-path ./repos/broken-app-6-optimized/Cargo.toml  \
    --output artifacts/generated/6.1_flamegraph_benches_baseline_sum_even.svg 
```

![](artifacts/committed/6.1_flamegraph_benches_hot_fix_sum_even.svg)

Наблюдаем над функцией `sum_even` «asm sysvec apic timer interrupt». Предполагаем, что эта полка возникла из-за `unsafe`. Перепишем `sum_even`, применив срезы и буферизацию без `unsafe`:

```rs
pub fn sum_even(values: &[i64]) -> i64 {
    let mut acc: i64 = 0;
    for &v in values {
        if (v & 1) == 0 { acc += v; }
    }
    acc
}
```

Сравним реализацию после hot-fix `sum_even_with_hot_fix`, оптимизированную по срезу `sum_even_new_optimized` и проверочную `sum_even_new_optimized` из reference-app:

```sh
cargo bench --bench criterion  \
    --manifest-path ./repos/broken-app-6-optimized/Cargo.toml -- sum_even
```

Получаем следующие результаты:

```
sum_even_new_optimized  time:   [4.7135 µs 4.7924 µs 4.8869 µs]
sum_even_with_hot_fix   time:   [11.479 µs 11.707 µs 11.972 µs]
sum_even_by_reference_app
                        time:   [6.4911 µs 6.5906 µs 6.7170 µs]
```

Полный лог в `artifacts/committed/6_1_1_optimyze_sum_even_get_criterion.log`. Самое быстрое решение (похоже, за счёт оптимизаций компилятора) — `sum_even_new_optimized`. Реализация из reference-app работает медленнее, т.к. создаётся объект итератора и его клонирование.

## 6.2 Оптимизация `average_positive`

Перепишем с использованием аккумуляторов без итераторов:

```rs 
pub fn average_positive(values: &[i64]) -> f64 {
    let mut acc: i64 = 0;
    let mut delimiter  = 0;
    for &v in values {
        if (v) > 0 { acc += v; delimiter +=1}
    }
    acc as f64 / delimiter as f64
}
```

Чтобы оценить эффективность, добавим реализацию из reference_app через итераторы, назвав функцию `average_positive_by_reference_app`. Оптимизированную функцию назовём `average_positive_new_optimized`, а текущую реализацию оставим под названием `average_positive_with_hot_fix`. Соответствующие тесты criterion назовём `bench_average_positive_new_optimized`, `bench_average_positive_by_reference_app` и `bench_average_positive_with_hot_fix`. Теперь построим criterion:

```sh
cargo bench --bench criterion  \
    --manifest-path ./repos/broken-app-6-optimized/Cargo.toml -- verage_positive
``` 

Получаем ускорение ×10 относительно `reference-app`:

```
bench_average_positive_with_hot_fix
                        time:   [63.795 ns 66.198 ns 68.756 ns]
bench_average_positive_by_reference_app
                        time:   [454.24 ns 485.74 ns 525.09 ns]
bench_average_positive_new_optimized
                        time:   [38.747 ns 40.228 ns 42.084 ns]
```

Лог запуска: `artifacts/committed/6_2_optimyze_average_positive_get_criterion.log`.

## 6.3 Оптимизация `normalize`

Как таковой «небрежной нормализации строки» не замечено, т.к. она просто неполная, если нужно ещё зачищать `\n` и `\t`. Двойные пробелы в текущей реализации удаляются корректно.

В файл `src/lib.rs` добавлена оригинальная функция `normalize` из `reference_app` под названием `normalize_by_reference_app`, ускоренная функция `normalize_new_optimized`, а текущую реализацию оставим под названием `normalize_with_hot_fix`. Для функций добавлены bench'и для сравнения скорости работы.

Попробуем следующую оптимизированную версию `normalize`:

```rs
/// b | 32 преобразует A..Z -> a..z.
///  Кириллица, эмодзи и другие UTF-8 символы останутся нетронутыми (их байты не попадают в диапазоны b' ', b'A'..=b'Z').
///
pub fn normalize(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b' ' | b'\t' | b'\n' => continue,
            // b | 32 эквивалентен to_ascii_lowercase(), но компилятор оптимизирует это в одну инструкцию
            b'A'..=b'Z' => out.push(b | 32),
            _ => out.push(b),
        }
    }
    // SAFETY: Мы модифицируем только ASCII-байты (0x00..0x7F) и не нарушаем 
    // структуру многобайтовых UTF-8 последовательностей. Валидность UTF-8 гарантирована.
    unsafe { String::from_utf8_unchecked(out) }
}
```

Запустим бенчмарки:

```sh
cargo bench --bench criterion  \
    --manifest-path ./repos/broken-app-6-optimized/Cargo.toml -- normalize
``` 

В выводе наблюдаем увеличение скорости в 3–5 раз по сравнению с reference-app:

```sh
...
bench_normalize_with_hot_fix
                        time:   [4.6812 µs 4.8052 µs 4.9419 µs]
bench_normalize_by_reference_app
                        time:   [2.9974 µs 3.1287 µs 3.2846 µs]
bench_normalize_new_optimized
                        time:   [676.01 ns 717.32 ns 767.68 ns]
```

### За счёт чего увеличена скорость?

| Применённые оптимизации | Что даёт |
|---|---|
| Работа с `&[u8]` вместо `chars()` | Пропускаем декодер UTF-8. ASCII-символы занимают 1 байт, проверка идёт напрямую. |
| Один проход (O(n)) | Не создаём промежуточные итераторы, аллокаторы или временные строки с последующим сбором вектора. |
| `b \| 32` | `b \| 32` вместо `to_ascii_lowercase()` |
| `Vec::with_capacity(src.len())` | Выделение памяти происходит один раз. Итераторные цепочки часто делают push без резерва, вызывая реаллокации. |
| LLVM Auto-Vectorization | В `--release` компилятор превращает этот цикл в SIMD-инструкции (AVX2/SSE4.1), обрабатывая по 16–32 байта за такт. |

Пример приведения символов ASCII к нижнему регистру с помощью `b|32`:

```rs
    let char_upper: u8 = b'A'; // 65 (0100 0001)
    let char_lower = char_upper | 32; // 65 | 32 = 97 (0110 0001)
```

Заглавные ASCII-буквы (`'A'`–`'Z'`) имеют коды 65–90. Строчные ASCII-буквы (`'a'`–`'z'`) имеют коды 97–122. Разница между ними — ровно 32 (2⁵). 6-й бит (считая с 0) у заглавных букв равен 0, а у строчных — 1.

## 6.4 Оптимизация `slow_dedup` и `slow_fib`

Позаимствуем `fast_dedup` и `fast_fib` из `reference_app`, т.к. дополнить к ним уже нечего, и построим бенчмарки:

```sh
cargo bench --bench criterion  \
    --manifest-path ./repos/broken-app-6-optimized/Cargo.toml -- fib
cargo bench --bench criterion  \
    --manifest-path ./repos/broken-app-6-optimized/Cargo.toml -- dedup
```

Подробные логи: `artifacts/committed/6_4_fast_dedub.log` и `artifacts/committed/6_4_fast_fib.log`. Результат:

```
slow_dedup_broken       time:   [35.525 ms 36.431 ms 37.450 ms]
bench_fast_dedup        time:   [401.36 µs 411.36 µs 423.23 µs]
slow_fib_broken         time:   [25.340 ms 26.327 ms 27.411 ms]
fast_fib_broken         time:   [68.196 ns 70.911 ns 73.991 ns]
```

Применим fast-функции для `demo`:

```rs
    let fib = algo::fast_fib(20);
    println!("fib(20): {}", fib);

    let uniq = algo::fast_dedup(&[1, 2, 2, 3, 1, 4, 4]);
    println!("dedup: {:?}", uniq);
```

# Шаг 7. Проверка «после»

<!--
git submodule add -b module_5/broken-app-6-optimized git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-7-final
pushd repos/broken-app-7-final
git checkout -b module_5/broken-app-7-final
git push --set-upstream origin module_5/broken-app-7-final
popd
 -->

На базе ветки `module_5/broken-app-6-optimized` оформим финальное решение в ветке `repos/broken-app-7-final` и `module_5/fix-solution-of-broken-app`. Контрольные проверки:

```sh
# miri
MIRIFLAGS="-Zmiri-backtrace=full -Awarnings"  cargo  miri test  -p is-not-broken-app
# asan
RUSTFLAGS="-Zsanitizer=address -Awarnings" cargo +nightly test -p is-not-broken-app \
            --target x86_64-unknown-linux-gnu
# tsan
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer -Awarnings" \
    cargo +nightly run --bin demo_for_threats -p is-not-broken-app \
		--target x86_64-unknown-linux-gnu
# бенчмарки
RUSTFLAGS="-Awarnings" cargo bench --bench criterion -p is-not-broken-app
```

Логи успешного финального прогона:
- `artifacts/generated/7_final_miri.log`
- `artifacts/generated/7_final_asan.log`
- `artifacts/generated/7_final_tsan.log`
- `artifacts/generated/7_final_benchmarks_criterion.log`

## 7.1 Резюме

В рамках работы было проведено:
- Быстрый фикс функций с помощью GDB;
- Восстановление работоспособности приложения `demo_for_threats`, использующего функции из `src/concurrency.rs`;
- Отлавливание срабатываний Miri, ASan/TSan и их фикс;
- Восстановление работоспособности бенчмарка criterion;
- Оптимизация функций `sum_even`, `normalize` и `average_positive` с ускорением их работы от 2× до 10× относительно `reference-app`;
- Построенный flamegraph и бенчмарки criterion для анализа улучшений.

# Troubleshooting

## Инструментированная сборка с Miri

```sh 
rustup component add miri
```

```sh
cargo +nightly miri setup  # один раз
cargo +nightly miri test counts_non_zero_bytes
```

## Инструментированная сборка с Valgrind

```sh 
sudo apt install valgrind
cargo install cargo-valgrind
```

```sh
valgrind --leak-check=full ./target/debug/your_binary_name
```

## Debug symbols

Проверка наличия символов:

```sh
objdump --dwarf=info target/debug/instrumentation-demo | less 
llvm-dwarfdump --debug-line target/debug/instrumentation-demo | head -n 40 
readelf -S target/debug/instrumentation-demo | grep debug 
```

## Git LFS track

```sh
git lfs env
git lfs track "artifacts/generated/*.csv"
git lfs track "./artifacts/**/*.csv"
git lfs ls-files
git show HEAD:artifacts/committed/reference-app-flamegraph-test-integration.svg
git show HEAD:artifacts/committed/reference-app-flamegraph-test-integration.svg
```

## Добавление reference-app как git submodule

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

## Установка flamegraph

```sh
cargo install rustfilt
cargo install demangle
git clone https://github.com/brendangregg/FlameGraph /tmp/FlameGraph
realpath ./FlameGraph
```

## Ошибка `feature edition2024 is required`

```
error: failed to parse manifest at `/home/svirsky/repos/yandex/reference-app/Cargo.toml`

Caused by:
  feature `edition2024` is required

  The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.75.0).
  Consider adding `cargo-features = ["edition2024"]` to the top of Cargo.toml (above the [package] table) to tell Cargo you are opting in to use this unstable feature.
```

### Fix

Install:

```sh
rustup default nightly
```

Add to `Cargo.toml`:

```toml
cargo-features = ["edition2024"]
```