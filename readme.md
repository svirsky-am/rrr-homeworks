# 0. Допущения

- для `race_increment`, read_after_sleep нет юниттестов , а reset_counter даже не описан не reference_app
- в тесте normalyze нет кейса на проверку `\n` `\t`. Поэтому правка выполнена чуть позже 
- в предоставленных broken_app  и reference_app не работают тесты criterion.
- нагядно оформить сбор логов только за счет коммитов и bash скриптовв оказалось тяжело. Был применен git submodules с отображением коммитов на каждом шаге ДЗ.

# Шаг 1. Ознакомление
## 1.1 Клонируйте оба проекта (broken-app, reference-app).
Сделаем доступными исходники для полученися логов и отчетов с помощью git submodules:
```sh
git submodule update --recursive
```
в папку `repos` будут вынружены экземпляры brocken_app в разных сосотяниях:
    - `reference-app` - ветка `module_5/reference-app` с ожидаемым поведением тестов `broker_app`;
    - `origin-of-broken-app`  - ветка `module_5/origin-of-broken-app` с оригинальными исходниками `broken-app`;
    - `origin-of-broken-app-fix-unit`  - ветка `module_5/origin-of-broken-app` оригинальные исходники `broken-app` с правкой unit-тестов;
    - `origin-of-broken-app-with-hot-fix` - ветка `module_5/origin-of-broken-app-with-hot-fix'` с простыми фиксами тестов без оптимизаций.
    - `roken-app-2.4-fix-leak-buffer-after-miri` - ветка `module_5/roken-app-2.4-fix-leak-buffer-after-miri'` с фиксом утечки `fn leak_buffer`.
<!-- - module_5/fix-solution-of-broken-appe -->

<!-- - module_5/reference-app-criterion-via-black-box -->
## 1.2. Фиксация падающих тестов в `origin-of-broken-app`

Проверим запуск тестов 
```sh
cargo check \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml
cargo test \
    --manifest-path ./repos/origin-of-broken-app/Cargo.toml
```

Получаем падение для `sums_even_numbers` :
```sh
cargo test \
		--manifest-path ./repos/origin-of-broken-app/Cargo.toml sums_even_numbers
```
получаем:
> thread 'sums_even_numbers' (2137351) panicked at src/lib.rs:11:29:
    unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice

и для `averages_only_positive`:
```sh
cargo test  \
        --manifest-path ./repos/origin-of-broken-app/Cargo.toml averages_only_positive
```
c ошибкой:
> thread 'averages_only_positive' (2138421) panicked at tests/integration.rs:36:5:
assertion failed: (broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

## 1.3 Воспролизведение ожидаемого результата
Все asert'ы от тестов прошли без паник: 
```sh
cargo test  \
        --manifest-path ./repos/reference-app/Cargo.toml
```
В stdout видим результат:
> test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
# Шаг 2. Поиск и исправление багов
## 2.1. Правка UB на для `sum_even`
Паника после запуска теста отсылает на к месту падения в исходниках `src/lib.rs:11 `.
Поставим точку останова на 11 ой строчке `src/lib.rs` и запустим дебагер. Видно что итератор следует до `idx=4`, т.к. values.len()=4, но индексация массива начинается с `0`, а последний элемент имеет индекс `3` и по хоршему надо выходить  при  `idx=3` чтобы не выйти за границы массива. 
![Состояние переменных перед вызоввом *values.get_unchecked(idx)](artifacts/committed/1.UB_sum_even.png)

### 2.1.1 Воспроизведение:
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
### 2.1.2 Быстрый фикс
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
Применим фикс на репозитории `repos/origin-of-broken-app-with-hot-fix` и запустим тест:
```sh
cargo test \
    --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml \
    sums_even_numbers
```
Тест пройден.
## 2.2.1 Правка фильтра позитивных значений для  `averages_only_positive`
Соберем тест:
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
Наблюдаем срабатывание asert`ов
 > assertion failed: (broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON
Таким образом нас интересует результат возвращаемый `broken_app::average_positive(&nums)`.
Запускаем дебагер:

```sh
rust-gdb $TEST_EXEC_PATH
```
Делаем точку останова на возврате функции 

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
Делаем вывод о том что элементы массива скорее всего сложились как (-5)+5+15.
Применим фильтр при складывании элементов:
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
## 2.3 Запуск cargo +nightly miri test для поиска UB.
```sh
MIRIFLAGS=-Zmiri-backtrace=full cargo  miri test --manifest-path ./repos/origin-of-broken-app-with-hot-fix/Cargo.toml
```
В логах (`artifacts/committed/2_3_miri_after_hot_fix.log`) в стеке видим 
>         7: broken_app::leak_buffer
>                at src/lib.rs:21:17: 21:31

Похоже мы вызываем Box::into_raw, но не освобождаем память через Box::from_raw.
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
Box::from_raw ожидает *mut [u8], поэтому нужно восстановить слайс правильной длины через slice::from_raw_parts_mut.

Проверяем фикс:
```sh
cargo  miri test --manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```
Тесты пройдены без ошибок (лог `artifacts/committed/2_4_fix_leak_buffer_after_miri.log`)

## 2.5. Запуск с Valgrind
### 2.5.1 asan
Проверим текущее состояние:
```sh
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```
По логу `artifacts/committed/mod5_2_5_1_valgrind_asan_first_one.log` ошибок не замечено.
### 2.5.2 tsan
```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer --leak-check=full --show-leak-kinds=all" cargo +nightly test \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```

По логу `artifacts/committed/mod5_2_5_2_valgrind_tsan_first_one.log` наблюдаем срабатывания tsan:
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
### 2.6. Анализ срабатываний tsan
Мы наблюдаем проблему с двумя потоками двух разных сессий. Починка тестового фремворка выходит за рамки ДЗ, поэтому вместо фикса попробуем убедиться что пробелема не в `lib.rs`.
Выполним аналогичный запуск для demo.rs:
```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly run --bin demo \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```
В логе `artifacts/committed/mod5_2_6_1_prove_leak_is_not_by_demo.log` срабатываний tsan не обнаружено.
Проверим через benches:
```sh
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly bench --bench baseline \
	--target x86_64-unknown-linux-gnu \
	--manifest-path ./repos/broken-app-2.4-fix-leak-buffer-after-miri/Cargo.toml
```
В логе `artifacts/committed/mod5_2_6_1_prove_leak_is_not_by_lib_rs___via_benches.log` срабатываний tsan не обнаружено.

> Вывод:
> Срабатывание tsan произошло в многопоточном тестовом фремворке. 
# Шаг 3. Подтверждение корректности
## 3.1 Добавление регрисионных тестов
Правки функций `sum_even` и `average_positive` покрываются имеющимися unit-тестами.

### 3.1.1 Для регрисионных unit-тестов `leak_buffer`

На базе `broken-app-2.4-fix-leak-buffer-after-miri` создадим ветку `broken-app-3.1.1-add-unit-tests-for-leak-buff`
<!-- git submodule add -b module_5/broken-app-2.4-fix-leak-buffer-after-miri git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-3.1.1-add-unit-tests-for-leak-buff -->
и добавим тесты:
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
В логе `artifacts/committed/broken-app-3.1.1-add-unit-tests-for-leak-buff.log` срабатываний tsan не обнаружено.
###  3.1.2 Фикс теста 


## 3.2 Контрольная проверка фиксов
Запустим имеющиеся тесты на ветке `broken-app-3.1.1-add-unit-tests-for-leak-buff`:
```sh
# miri
cargo  miri test --manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml
# asan
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test  \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml
# tsan
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer " cargo +nightly run --bin demo \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml
```
Результат в `artifacts/committed/mod5_3_1_2_check_after_hot_fix.log`
## 3.3 Дополнение имеющихся unit-тестов
Анализируя тесты , можно заметить что: 
- тест `normalize_simple` для функции `normalize` не покрывает кейсов , когда тестовая строка содержит в себе табуляции `\t` и переносы строк `\n`;
- нет тестов для `race_increment`.  
- нет тестов для `read_after_sleep` 
- нет тестов для `reset_counter`.
Сделаем правки на ветке `module_5/broken-app-3.3-extra-tests-for-normalyze-and-threat`, созданной на базе ветки `broken-app-3.1.1-add-unit-tests-for-leak-buff`
<!-- git submodule add -b module_5/broken-app-3.1.1-add-unit-tests-for-leak-buff git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-3.3-extra-tests-for-normalyze-and-threat -->
## 3.3.1 Тест `normalize_simple`
Для проверки `normalize` в соответсвии с описанием к функции нужно добавить проверяемые символы в проверочную строку:
```rs
#[test]
fn normalize_simple() {
    assert_eq!(normalize(" H  e\n\nllo                       Wo\t\t\t\t\t\t\t\t\t\t\trld "), "helloworld");
}
```
и сразу применим фикс к функции:
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

## 3.3.2 Тесты для функций  `race_increment`, `read_after_sleep` и `reset_counter`

Портируем имеющийся тест `race_increment_is_correct` из reference_app и запустим его:
```sh 
cargo test \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml -- race_increment_is_correct
```
и не увидим гонки
> running 1 test
> test race_increment_is_correct ... ok

Для запуска tsan оформим тестовое приложение `demo_for_threats`, использующее эти функции, т.к тестовый фреймворк имеет свои ошибки при запуске tsan.
```rs
use broken_app::{concurrency};

fn main() {

    let total = concurrency::race_increment(1_000, 4);
    println!("total: {:?}", &total);
    concurrency::read_after_sleep();
    concurrency::reset_counter();
    println!("total: {:?}", &total);
    
}
```
Запусти несколько раз:

```sh
cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3-extra-tests-for-normalyze-and-threat/Cargo.toml 
```
Каждый раз получаем рандомные значения 
> total: 3131
> total: 3131

> total: 3856
> total: 3856

Запустим tsan на этом приложении:
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
Подробности в логе `artifacts/committed/broken-app-3.3-run-bin-demo_for_threats-via-tsan.log`.
Коммитимся и фиксим.

## 3.3.3 Фиксы для функций  `race_increment`, `read_after_sleep` и `reset_counter` и приложения `demo_for_threats`
Фикс сделаем на базе ветки `module_5/broken-app-3.3-extra-tests-for-normalyze-and-threat`
<!--
git submodule add -b module_5/broken-app-3.3-extra-tests-for-normalyze-and-threat git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats
pushd repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats
git checkout -b module_5/broken-app-3.3.3-fix-tsan-for-demo-for-threats
git push --set-upstream origin module_5/broken-app-3.3.3-fix-tsan-for-demo-for-threats
popd
 -->
```sh
cargo +nightly run --bin demo_for_threats \
		--target x86_64-unknown-linux-gnu \
		--manifest-path ./repos/broken-app-3.3.3-fix-tsan-for-demo-for-threats/Cargo.toml 
```







# Шаг 4. Поиск узких мест
## 4.1 Построение flamegraph для demo
На базе ветки с хотфиксами   оригинального 



# Шаг 5. Бенчмарки до оптимизации
## 5.1 Настройка бенчмарков для broken_app и reference_app
### 5.1.1. Праввка вызова criterion в `broken_app`
Оригинальном broken-app с hot-фиксами не работает бенчмарк criterion
```sh
cargo bench  --bench criterion  \
		--manifest-path ./repos/broken-app-3.1.1-add-unit-tests-for-leak-buff/Cargo.toml
```


Выполним профилирование после горячих фиксов на ветке `broken-app-3.1.1-add-unit-tests-for-leak-buff`, полученной на базе ветки `broken-app-4.1.1-fix-criterion`.
<!-- git submodule add -b module_5/broken-app-3.1.1-add-unit-tests-for-leak-buff git@github.com:svirsky-am/rrr-homeworks.git repos/broken-app-5.1.1-fix-criterion -->




## Шаг 6. Оптимизация
## 6.1 Микро оптимизация `sum_even`
Самое быстрое решение (похоже за счет оптимизаций компилдятора) ~546ns против   : ~164.27µs в reference app
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

# 2. Правка обчног запуска cargo test

Перед запуском тестов miri и Valgrind починим обычный запуск unit-тестов.
```sh
cargo test
```
имеем срабатывания асерта: 
```
---- averages_only_positive stdout ----

thread 'averages_only_positive' (1479910) panicked at tests/integration.rs:36:5:
assertion failed: (broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

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
Чтобы оценить эффективность , добавим реализацию из reference_app через итераторы, назвав функцию `average_positive_by_reference_app` и соответсвующий тесты criterion `bench_average_positive_by_reference_app` и `bench_average_positive`.  Теперь построим criterion :
```sh
cargo bench  --bench criterion
``` 

 Получяаем ускорение x10 относительно `reference-app`:
 ```
 bench_average_positive  time:   [40.005 ns 40.636 ns 41.329 ns]
 bench_average_positive_by_reference_app
                        time:   [636.34 ns 641.22 ns 647.17 ns]
 ```




# 3. Анализ fn normalize

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
или отчет html:
```sh 
xdg-open target/criterion/report/index.html
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

# 3. Инструментированная сборка с Miri
```sh 
rustup component add miri
```
```sh
cargo +nightly miri setup  # один раз
cargo +nightly miri test counts_non_zero_bytes
```

# 3. Инструментированная сборка с valgrind
```sh 

sudo apt install valgrindy

cargo install cargo-valgrind
```



```sh

valgrind --leak-check=full ./target/debug/your_binary_name
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

