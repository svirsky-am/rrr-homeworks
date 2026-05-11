pub mod algo;
pub mod concurrency;

/// Сумма чётных значений.
/// Здесь намеренно используется `get_unchecked` с off-by-one,
/// из-за чего возникает UB при доступе за пределы среза.
pub fn sum_even(values: &[i64]) -> i64 {

    // // Быстрое решение 1
    // let mut acc = 0;
    // unsafe {
    //     for idx in 0..=values.len() - 1  {
    //         let v = *values.get_unchecked(idx);
    //         if v % 2 == 0 {
    //             acc += v;
    //         }
    //     }
    // }
    // acc

    // // Решение похожее на reference-app 1
    //values.iter().filter(|v| *v % 2 == 0).sum()


    // // Самое быстрое решение (похоже за счет оптимизаций компилдятора) ~546ns против   sum_even: ~164.27µs в reference app

    let mut acc: i64 = 0;
    for &v in values {
        if (v & 1) == 0 { acc += v; }
    }
    acc
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
        // утечка: не вызываем Box::from_raw(raw);
    }
    count
}

/// Небрежная нормализация строки: удаляем пробелы и приводим к нижнему регистру,
/// но игнорируем повторяющиеся пробелы/табуляции внутри текста.
/// 
pub fn normalize(input: &str) -> String {
    input.replace(' ', "").to_lowercase()
    // input.trim().to_lowercase()
    // input.trim().to_lowercase()
}

pub fn normalize_by_reference_app(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<String>()
        .to_lowercase()
} 

/// b | 32 преобразует A..Z → a..z.
///  Кириллица, эмодзи и другие UTF-8 символы останутся нетронутыми (их байты не попадают в диапазоны b' ', b'A'..=b'Z').
///
pub fn normalize_faster_new_alt(src: &str) -> String {
    let mut out = Vec::with_capacity(src.len());
    for &b in src.as_bytes() {
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



/// Логическая ошибка: усредняет по всем элементам, хотя требуется учитывать
/// только положительные. Деление на длину среза даёт неверный результат.
pub fn average_positive(values: &[i64]) -> f64 {
    let sum: i64 = values.iter().sum();
    if values.is_empty() {
        return 0.0;
    }
    sum as f64 / values.len() as f64
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
