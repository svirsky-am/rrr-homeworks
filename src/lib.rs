pub mod algo;
pub mod concurrency;

/// с hot fix
pub fn sum_even_with_hot_fix(values: &[i64]) -> i64 {
    let mut acc = 0;
    unsafe {
        for idx in 0..=values.len() - 1 {
            let v = *values.get_unchecked(idx);
            if v % 2 == 0 {
                acc += v;
            }
        }
    }
    acc
}

/// Оптимизировавнный вариант 
pub fn sum_even_new_optimized(values: &[i64]) -> i64 {
    let mut acc: i64 = 0;
    for &v in values {
        if (v & 1) == 0 { acc += v; }
    }
    acc
}

pub fn sum_even(values: &[i64]) -> i64 {
    sum_even_new_optimized(values)
}


/// Реализация из reference-app работает модленее т.к. создается объект итератора и его копирование .
/// Скорее всего тут еще два прохода по итератору: один фильтр, второй sum
pub fn sum_even_by_reference_app(values: &[i64]) -> i64 {
    values.iter().copied().filter(|v| v % 2 == 0).sum()
}


/// Подсчёт ненулевых байтов. Буфер намеренно не освобождается,
/// что приведёт к утечке памяти (Valgrind это покажет).
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
        let _ = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(raw, len)) };
    }
    count
}

/// Небрежная нормализация строки: удаляем пробелы и приводим к нижнему регистру,
/// но игнорируем повторяющиеся пробелы/табуляции внутри текста.
/// 
/// hot fix
pub fn normalize(input: &str) -> String {
    input.replace(' ', "").replace('\n', "").replace('\t', "").to_lowercase()
}

/// с hot fix
/// 
pub fn average_positive_with_hot_fix(values: &[i64]) -> f64 {
    let sum: i64 = values.iter().filter(|&&x| x > 0).sum();
    if values.is_empty() {
        return 0.0;
    }
    sum as f64 / values.iter().filter(|&&x| x > 0).count() as f64
}

/// Корректное усреднение только положительных чисел. (реализация reference-app)
pub fn average_positive_by_reference_app(values: &[i64]) -> f64 {
    let positives: Vec<i64> = values.iter().copied().filter(|v| *v > 0).collect();
    if positives.is_empty() {
        return 0.0;
    }
    let sum: i64 = positives.iter().sum();
    sum as f64 / positives.len() as f64
}

pub fn average_positive_new_optimized(values: &[i64]) -> f64 {
    let mut acc: i64 = 0;
    let mut delimiter  = 0;
    for &v in values {
        if (v) > 0 { acc += v; delimiter +=1}
    }
    acc as f64 / delimiter as f64
}

//  обертка для bin demo  и юнит тестов
pub fn average_positive(values: &[i64]) -> f64 {
    average_positive_new_optimized(values)
}



/// Use-after-free: возвращает значение после освобождения бокса.
/// UB, проявится под ASan/Miri.
pub unsafe fn use_after_free() -> i32 {
    let b = Box::new(42_i32);
    let raw = Box::into_raw(b);
    let val = *raw;
    drop(Box::from_raw(raw));
    val + *raw
}
