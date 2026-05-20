
use broken_app::{concurrency};

fn main() {
    println!("Запуск race_increment(1_000, 4)...");
    
    // Ожидаемый результат: 1_000 * 4 = 4_000
    let total = concurrency::race_increment(1_000, 4);
    println!("После инкремента: total = {}", total);
    
    // Читаем после задержки — теперь это безопасно
    let after_sleep = concurrency::read_after_sleep();
    println!("После sleep: counter = {}", after_sleep);
    
    // Сбрасываем и проверяем
    concurrency::reset_counter();
    let after_reset = concurrency::read_after_sleep();
    println!("После reset: counter = {}", after_reset);
    
    // Финальная проверка
    assert_eq!(total, 4_000, "Ожидалось 4000, получено {}", total);
    assert_eq!(after_reset, 0, "После сброса счётчик должен быть 0");
    
    println!("Все проверки пройдены!");
}