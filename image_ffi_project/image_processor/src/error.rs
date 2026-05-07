use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Plugin loading error: {0}")]
    PluginLoadError(#[from] libloading::Error),
    #[error("Plugin symbol 'process_image' not found")]
    PluginSymbolNotFound,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parameter error: {0}")]
    ParamError(String),
    #[error("Failed to reconstruct image buffer (size mismatch)")]
    BufferMismatch,
}
