

# image_ffi_project

Обработчик изображений с динамическими FFI-плагинами.

## Architecture
- `image_processor`: CLI-приложение, обрабатывающее PNG-изображения с делегацией обработки динамически подключаемым `.so`/`.dll` плагинам.
- `mirror_plugin`: поворачивает изображение  вертикально/горизонтально.
- `blur_plugin`: Применияет блюр.

## Build & Run
1. **Build the project:**
```sh
cargo build -p  "mirror_plugin"
cargo build -p  "blur_plugin"
cargo build -p  "image_processor"
```
Собранный бинарный файл и оба плагина сохранены в `target/debug/`.
2. Запуск тестов:
```sh
cargo test -p  "mirror_plugin" --release
cargo test -p  "blur_plugin" --release
RUST_LOG=debug cargo test -p  "image_processor" --test integration -- --nocapture
```
3. Запуск утилиты в debug:
```sh
cargo run --bin image_processor -- \
  --input image_ffi_project/test/input.png \
  --output output/output_mirror.png \
  --plugin image_ffi_project/mirror_plugin \
  --params image_ffi_project/test/params_mirror.json \
  --plugin-path target/debug
cargo run --bin image_processor -- \
  --input image_ffi_project/test/input.png \
  --output output/output_blur.png \
  --plugin image_ffi_project/blur_plugin \
  --params image_ffi_project/test/params_blur.json \
  --plugin-path target/debug
```
# Plugin API
Все плагины должны экспортироваться как C-совместимые функции:
```c
void process_image(uint32_t width, uint32_t height, uint8_t* rgba_data, const char* params);
```
- width, height: Image dimensions.
- rgba_data: Pointer to width * height * 4 byte buffer (in-place modification).
- params: JSON string with plugin-specific configuration.
# Сбор покрытия интеграционных тетсов
Установка llvm-cov для инструментированной сборки и теста :
```sh
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```
Запуск теста и постоение отчета:
```sh
cargo llvm-cov -p  "image_processor"
cargo llvm-cov test --test integration_cov -p  "image_processor" -p  "blur_plugin" -p  "mirror_plugin" --html
```
# Safety Guarantees
- Все вызовы FFI строго обёрнуты в блоки `unsafe` с явными проверками времён жизни и границ буферов.
- Крейт `libloading` удерживает динамическую библиотеку в памяти до тех пор, пока экземпляр структуры `Plugin` не будет уничтожен.
- Тип `CString` обеспечивает безопасную передачу строк параметров с нулевым терминатором и автоматическую очистку памяти после использования.
- Все операции работы с файлами возвращают тип `Result`, что исключает панику при отсутствии или повреждении входных данных.
- Встроенные юнит-тесты в плагинах проверяют математику преобразований без необходимости запуска основного бинарника.
## Track git lfs 
```sh
git lfs track --filename "image_ffi_project/test/input.png"
```