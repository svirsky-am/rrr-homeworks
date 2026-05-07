// image_processor/src/main.rs
use clap::Parser;
use image::RgbaImage;
use std::fs;
use std::path::PathBuf;
use log::{info, error};

mod error;
mod plugin_loader;

use error::AppError;
use plugin_loader::Plugin;

#[derive(Parser)]
#[command(name = "image_ffi_processor")]
#[command(about = "Image processor with dynamic FFI plugins")]
struct Cli {
    /// Path to input PNG image
    #[arg(long)]
    input: PathBuf,

    /// Path to output PNG image
    #[arg(long)]
    output: PathBuf,

    /// Plugin name (without extension, e.g., 'invert')
    #[arg(long)]
    plugin: String,

    /// Path to text/JSON file with processing parameters
    #[arg(long)]
    params: PathBuf,

    /// Directory containing compiled plugins
    #[arg(long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn run() -> Result<(), AppError> {
    
    env_logger::Builder::from_env(
        env_logger::Env::default().filter_or("RUST_LOG", "info")
    )
    .target(env_logger::Target::Stdout)
    .init();

    let args = Cli::parse();

    // Валидация входных файлов
    if !args.input.exists() {
        return Err(AppError::FileNotFound(args.input));
    }
    if !args.params.exists() {
        return Err(AppError::FileNotFound(args.params));
    }

    let params_content = fs::read_to_string(&args.params)?;
    info!("Parameters loaded from {:?}", args.params);

    // Загрузка и преобразование изображения
    info!("Opening image: {:?}", args.input);
    let img = image::open(&args.input)?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let expected_len = (width * height * 4) as usize;

    let mut pixels: Vec<u8> = rgba_img.into_raw();
    if pixels.len() != expected_len {
        return Err(AppError::BufferMismatch);
    }
    info!("Image loaded: {}x{} ({} bytes)", width, height, expected_len);

    // Динамическая загрузка плагина
    info!("Loading plugin '{}' from {:?}", args.plugin, args.plugin_path);
    let plugin = Plugin::load(&args.plugin, &args.plugin_path)?;

    // Обработка
    info!("Processing image with plugin...");
    plugin.process(width, height, &mut pixels, &params_content)?;

    // Сохранение результата
    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() {
            info!("Ensuring output directory exists: {:?}", parent);
            fs::create_dir_all(parent)?;
        }
    }
    info!("Saving result to {:?}", args.output);

    let out_img = RgbaImage::from_raw(width, height, pixels)
        .ok_or(AppError::BufferMismatch)?;
    out_img.save(&args.output)?;

    info!("Processing completed successfully!");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        error!("Application failed: {}", e);
        std::process::exit(1);
    }
}