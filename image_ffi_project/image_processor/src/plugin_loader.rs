// image_processor/src/plugin_loader.rs
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;

use crate::error::AppError;

// Сигнатура функции плагина, соответствующая C API
pub type ProcessImageFn = unsafe extern "C" fn(u32, u32, *mut u8, *const c_char);

pub struct Plugin {
    // Храним Library, чтобы предотвратить выгрузку библиотеки до завершения работы
    _lib: Library,
    process_image: ProcessImageFn,
}

impl Plugin {
    pub fn load(plugin_name: &str, plugin_path: &Path) -> Result<Self, AppError> {
        // 1. Очищаем имя от возможных путей и расширений
        let clean_name = std::path::Path::new(plugin_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(plugin_name);

        // 2. Определяем расширение под ОС
        let ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        // 3. Формируем имя файла по конвенции cdylib: lib{name}.ext
        let lib_filename = format!("lib{}.{}", clean_name, ext);
        let lib_path = plugin_path.join(&lib_filename);

        if !lib_path.exists() {
            return Err(AppError::FileNotFound(lib_path));
        }

        // SAFETY: Загрузка динамической библиотеки небезопасна, так как может выполнять
        // произвольный код при инициализации (статические конструкторы). Мы контролируем
        // путь к плагину и ожидаем только корректные cdylib-крейты.
        let lib = unsafe { Library::new(&lib_path)? };

        // SAFETY: Мы явно запрашиваем символ с известной сигнатурой. Библиотека гарантированно
        // остаётся в памяти, пока жив `lib`. Указатель валиден до момента Drop.
        let symbol: Symbol<ProcessImageFn> =
            unsafe { lib.get::<ProcessImageFn>(b"process_image")? };

        // Извлекаем сырой указатель на функцию. Библиотека останется загруженной благодаря _lib.
        let process_image = *symbol;

        Ok(Plugin {
            _lib: lib,
            process_image,
        })
    }

    pub fn process(
        &self,
        width: u32,
        height: u32,
        rgba_data: &mut [u8],
        params: &str,
    ) -> Result<(), AppError> {
        // Безопасное преобразование в C-строку
        let c_params = CString::new(params)
            .map_err(|_| AppError::ParamError("Params contain invalid null byte".into()))?;

        // SAFETY:
        // 1. `self.process_image` указывает на валидную функцию из загруженной библиотеки.
        // 2. `rgba_data` — это валидный изменяемый срез с точной длиной `width * height * 4`.
        // 3. `c_params` — валидная null-terminated строка, живущая до конца функции.
        // 4. Плагин работает синхронно и не удерживает указатели после возврата.
        unsafe {
            (self.process_image)(width, height, rgba_data.as_mut_ptr(), c_params.as_ptr());
        }
        // CString и срез данных живут до конца этой функции, что гарантирует безопасность
        Ok(())
    }
}
