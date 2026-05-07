use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Failed to resolve workspace root")
        .parent()
        .expect("Failed to resolve workspace root")
        .to_path_buf()
}

// Запуск строго из корня проекта!
#[test]
fn integration_test_mirror_and_blur() {
    // Cargo автоматически подставляет путь к бинарнику.
    // При запуске `cargo llvm-cov` это будет инструментированная версия.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = workspace_root();
    let target_dir = root.join("target");
    let binary = PathBuf::from(&target_dir)
        .join("llvm-cov-target/debug/image_processor")
        .to_string_lossy()
        .into_owned();
    dbg!(&binary);

    let plugin_path = &target_dir.join("debug");

    dbg!(&root);

    let input = root.join("image_ffi_project/test/input.png");
    let params_mirror = root.join("image_ffi_project/test/params_mirror.json");
    let params_blur = root.join("image_ffi_project/test/params_blur.json");

    assert!(
        input.exists(),
        "Test image not found at ../image_ffi_project/test/input.png"
    );
    assert!(params_mirror.exists(), "Mirror params not found");
    assert!(params_blur.exists(), "Blur params not found");

    // Временная директория для вывода
    let out_dir = env::temp_dir().join("image_ffi_cov_test");
    fs::create_dir_all(&out_dir).expect("Failed to create temp output dir");
    let out_mirror = out_dir.join("output_mirror.png");
    let out_blur = out_dir.join("output_blur.png");

    let run_processor = |output: &PathBuf, plugin: &str, params: &PathBuf| {
        Command::new(&binary)
            .args([
                "--input",
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--plugin",
                plugin,
                "--params",
                params.to_str().unwrap(),
                "--plugin-path",
                plugin_path.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to spawn image_processor")
    };

    // 2. Тест плагина зеркального отражения
    println!("Running mirror plugin test...");
    let status = run_processor(&out_mirror, "mirror_plugin", &params_mirror);
    assert!(status.success(), "Mirror plugin execution failed");
    assert!(out_mirror.exists(), "Mirror output file was not created");
    assert!(
        out_mirror.metadata().map(|m| m.len()).unwrap_or(0) > 0,
        "Mirror output is empty"
    );

    // 3. Тест плагина размытия
    println!("Running blur plugin test...");
    let status = run_processor(&out_blur, "blur_plugin", &params_blur);
    assert!(status.success(), "Blur plugin execution failed");
    assert!(out_blur.exists(), "Blur output file was not created");
    assert!(
        out_blur.metadata().map(|m| m.len()).unwrap_or(0) > 0,
        "Blur output is empty"
    );

    fs::remove_dir_all(&out_dir).ok();
    println!("✅ Integration tests passed with coverage!");
}
