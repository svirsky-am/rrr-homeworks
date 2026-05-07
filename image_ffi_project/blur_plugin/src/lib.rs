use serde::Deserialize;
use std::os::raw::c_char;

#[derive(Deserialize)]
struct BlurParams {
    #[serde(default = "default_radius")]
    radius: usize,
    #[serde(default = "default_iterations")]
    iterations: usize,
}
fn default_radius() -> usize { 2 }
fn default_iterations() -> usize { 1 }

#[no_mangle]
pub extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const c_char,
) {
    let width = width as usize;
    let height = height as usize;
    let len = width * height * 4;
    let buffer = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };
    let params_str = unsafe {
        std::ffi::CStr::from_ptr(params).to_str().unwrap_or("{}")
    };

    let config = match serde_json::from_str::<BlurParams>(params_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[blur_plugin] Invalid params JSON: {}", e);
            return;
        }
    };

    // Временный буфер для избежания конфликта чтения/записи при in-place модификации
let mut src = buffer.to_vec();
    let mut dst = vec![0u8; len];

    for _ in 0..config.iterations {
        apply_box_blur(&src, &mut dst, width, height, config.radius);
        // Меняем местами указатели на данные: dst становится источником для следующей итерации
        std::mem::swap(&mut src, &mut dst);
    }
    
    // Копируем финальный результат обратно в исходный буфер
    buffer.copy_from_slice(&src);
}

fn apply_box_blur(src: &[u8], dst: &mut [u8], w: usize, h: usize, radius: usize) {
    for y in 0..h {
        for x in 0..w {
            let mut r_acc = 0u32;
            let mut g_acc = 0u32;
            let mut b_acc = 0u32;
            let mut a_acc = 0u32;
            let mut count = 0u32;

            // Ограничиваем область просмотра границами изображения
            let y_start = y.saturating_sub(radius);
            let y_end = (y + radius).min(h - 1);
            let x_start = x.saturating_sub(radius);
            let x_end = (x + radius).min(w - 1);

            for ny in y_start..=y_end {
                for nx in x_start..=x_end {
                    let idx = (ny * w + nx) * 4;
                    r_acc += src[idx] as u32;
                    g_acc += src[idx + 1] as u32;
                    b_acc += src[idx + 2] as u32;
                    a_acc += src[idx + 3] as u32;
                    count += 1;
                }
            }

            let center = (y * w + x) * 4;
            dst[center] = (r_acc / count) as u8;
            dst[center + 1] = (g_acc / count) as u8;
            dst[center + 2] = (b_acc / count) as u8;
            dst[center + 3] = (a_acc / count) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_uniform_buffer(w: usize, h: usize, val: u8) -> Vec<u8> {
        vec![val; w * h * 4]
    }

    #[test]
    fn test_blur_preserves_uniform() {
        let w = 10;
        let h = 10;
        let mut src = create_uniform_buffer(w, h, 100);
        let mut dst = vec![0; w * h * 4];
        apply_box_blur(&src, &mut dst, w, h, 3);
        // Размытие однородного поля должно оставить значения неизменными
        assert_eq!(dst, src);
    }

    #[test]
    fn test_blur_averages_correctly() {
        // 3x3 изображение с разными значениями в красном канале
        let w = 3; let h = 3;
        let mut buf = vec![0u8; w * h * 4];
        // Устанавливаем R каналы: 0..8
        for i in 0..w*h { buf[i*4] = i as u8; }
        
        let mut dst = vec![0; w*h*4];
        apply_box_blur(&buf, &mut dst, w, h, 1);
        
        // Центральный пиксель (1,1) должен усреднить 0..8 = 36/9 = 4
        let center_r = dst[(1*w + 1)*4];
        assert_eq!(center_r, 4);
    }
}