//! Logging utilities for the streaming quotes project.
//!
//! Provides centralized logging initialization with configurable log levels
//! via the `RUST_LOG` environment variable.

pub use log::{debug, error, info, trace, warn};

/// Initializes the logging system.
///
/// This function should be called once at the start of the application.
/// Log level is controlled by the `RUST_LOG` environment variable.
///
/// # Examples
///
/// ```no_run
/// streaming_quotes_project::logging::init_logger();
/// ```
///
/// # Environment Variables
///
/// - `RUST_LOG=info` - Show info and above
/// - `RUST_LOG=debug` - Show debug and above
/// - `RUST_LOG=streaming_quotes_project=debug` - Debug for this crate only
pub fn init_logger() {
    let _ = env_logger::builder()
        .format_timestamp_millis()
        .target(env_logger::Target::Stdout)
        .parse_default_env()
        .try_init();
    info!("Logging initialized");
}
