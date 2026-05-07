use std::path::PathBuf;
use std::process::Command;
use std::env;
use std::fs;

/// Вычисляет корень workspace на основе манифеста текущего крейта
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Failed to resolve workspace root")
        .to_path_buf()
}


// Запуск строго из корня проекта!

#[test]
fn integration_test_mirror_and_blur() {
    let root = workspace_root();
    let binary = root.join("../target/debug/image_processor");
    let plugin_path = root.join("../target/debug");
    
    // Пути к тестовым данным
    let input = root.join("../image_ffi_project/test/input.png");
    let params_mirror = root.join("../image_ffi_project/test/params_mirror.json");
    let params_blur = root.join("../image_ffi_project/test/params_blur.json");

    // Предварительные проверки (чтобы тест не падал с cryptic error, если файлов нет)
    assert!(binary.exists(), "Binary not found. Run `cargo build` first.");
    assert!(input.exists(), "Test image not found at image_ffi_project/test/input.png");
    assert!(params_mirror.exists(), "Mirror params not found at image_ffi_project/test/params_mirror.json");
    assert!(params_blur.exists(), "Blur params not found at image_ffi_project/test/params_blur.json");

    // 1. Сборка плагинов
    println!("🔨 Building plugins...");
    let build_status = Command::new("cargo")
        .args(["build", "-p", "mirror_plugin", "-p", "blur_plugin"])
        .status()
        .expect("Failed to execute cargo build");
    assert!(build_status.success(), "Plugin build failed");

    // Временная директория для вывода (чтобы не мусорить в репозитории)
    let out_dir = env::temp_dir().join("output").join("image_ffi_integration_test");
    fs::create_dir_all(&out_dir).expect("Failed to create temp output dir");
    let out_mirror = out_dir.join("output_mirror.png");
    let out_blur = out_dir.join("output_blur.png");

    // Хелпер для запуска процессора
    let run_processor = |output_path: &PathBuf, plugin: &str, params_path: &PathBuf| {
        Command::new(&binary)
            .args([
                "--input", input.to_str().unwrap(),
                "--output", output_path.to_str().unwrap(),
                "--plugin", plugin,
                "--params", params_path.to_str().unwrap(),
                "--plugin-path", plugin_path.to_str().unwrap(),
            ])
            // Отключаем логи во время теста, если требуется не засорять вывод cargo test
            // .env("RUST_LOG", "off")
            .status()
            .expect("Failed to spawn image_processor")
    };

    // 2. Тест плагина зеркального отражения
    println!("🪞 Running mirror plugin test...");
    let status = run_processor(&out_mirror, "mirror_plugin", &params_mirror);
    assert!(status.success(), "Mirror plugin execution failed");
    assert!(out_mirror.exists(), "Mirror output file was not created");
    assert!(out_mirror.metadata().map(|m| m.len()).unwrap_or(0) > 0, "Mirror output is empty");

    // 3. Тест плагина размытия
    println!("🌫️ Running blur plugin test...");
    let status = run_processor(&out_blur, "blur_plugin", &params_blur);
    assert!(status.success(), "Blur plugin execution failed");
    assert!(out_blur.exists(), "Blur output file was not created");
    assert!(out_blur.metadata().map(|m| m.len()).unwrap_or(0) > 0, "Blur output is empty");

    // Очистка временных файлов
    // fs::remove_dir_all(&out_dir).ok();
    println!("✅ All integration tests passed!");
}