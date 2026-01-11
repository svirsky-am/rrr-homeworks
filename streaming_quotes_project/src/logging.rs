#[cfg(feature = "logging")]
pub use log::{debug, error, info, trace, warn};

// Макросы-заглушки, когда фича logging отключена
#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

// Функция инициализации логирования
#[cfg(feature = "logging")]
pub fn init_logger() {
    env_logger::init();
    info!("Логирование инициализировано");
}

#[cfg(not(feature = "logging"))]
pub fn init_logger() {
    // Ничего не делаем, когда логирование отключено
}
