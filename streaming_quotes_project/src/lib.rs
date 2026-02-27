
pub mod metrics;
// pub mod receiver;
pub mod sender;

pub use metrics::RoomMetrics;
// pub use receiver::ServerUDPReceiver;
// pub use sender::ClientUDPsender; 

// Условно компилируем модуль логирования
#[cfg(feature = "logging")]
pub mod logging;

#[cfg(not(feature = "logging"))]
mod logging;



// Реэкспортируем макросы логирования
#[cfg(feature = "logging")]
pub use logging::{debug, error, info, trace, warn, init_logger};

#[cfg(not(feature = "logging"))]
pub use logging::{init_logger};


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
