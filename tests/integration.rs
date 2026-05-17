use broken_app::{algo, leak_buffer, normalize, sum_even};

#[test]
fn sums_even_numbers() {
    let nums = [1, 2, 3, 4];
    // Ожидаем корректное суммирование: 2 + 4 = 6.
    assert_eq!(sum_even(&nums), 6);
}

#[test]
fn counts_non_zero_bytes() {
    let data = [0_u8, 1, 0, 2, 3];
    assert_eq!(leak_buffer(&data), 3);
}

#[test]
fn test_leak_buffer_zero_vs_nonzero_distinction() {
    let mut input = vec![0u8; 10_000];
    // Каждое 10-е значение ненулевое
    for i in (0..input.len()).step_by(10) {
        input[i] = 42;
    }
    assert_eq!(leak_buffer(&input), 1_000);
    // далее просто тесты
    assert_eq!(leak_buffer(&[0, 0, 0]), 0);
    assert_eq!(leak_buffer(&[1, 1, 1]), 3);
    assert_eq!(leak_buffer(&[0, 1, 0, 1, 0]), 2);
    assert_eq!(leak_buffer(&[255, 0, 128, 0, 1]), 3);
    let full = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    // Берём срез, начинающийся с нечётного индекса
    assert_eq!(leak_buffer(&full[1..6]), 5); // [1,2,3,4,5] → все ненулевые
    //Большие данные (проверка производительности и переполнений)

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


#[test]
fn dedup_preserves_uniques() {
    let uniq = algo::slow_dedup(&[5, 5, 1, 2, 2, 3]);
    assert_eq!(uniq, vec![1, 2, 3, 5]); // порядок и состав важны
}

#[test]
fn fib_small_numbers() {
    assert_eq!(algo::slow_fib(10), 55);
}

#[test]
fn normalize_simple() {
    assert_eq!(normalize(" H  e\n\nllo                       Wo\t\t\t\t\t\t\t\t\t\t\trld "), "helloworld");
}

#[test]
fn averages_only_positive() {
    let nums = [-5, 5, 15];
    // Ожидается (5 + 15) / 2 = 10, но текущая реализация делит на все элементы.
    assert!((broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON);
}
