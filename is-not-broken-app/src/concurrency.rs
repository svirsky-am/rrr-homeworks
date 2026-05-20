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