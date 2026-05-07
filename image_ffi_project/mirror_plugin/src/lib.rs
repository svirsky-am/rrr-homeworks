use serde::Deserialize;
use std::os::raw::c_char;

#[derive(Deserialize)]
struct MirrorParams {
    #[serde(default)]
    horizontal: bool,
    #[serde(default)]
    vertical: bool,
}

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
    
    // Безопасное создание слайса на основе переданного указателя
    let buffer = unsafe { std::slice::from_raw_parts_mut(rgba_data, len) };
    let params_str = unsafe {
        std::ffi::CStr::from_ptr(params).to_str().unwrap_or("{}")
    };

    let config = match serde_json::from_str::<MirrorParams>(params_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[mirror_plugin] Invalid params JSON: {}", e);
            return;
        }
    };

    if config.horizontal {
        flip_horizontal(buffer, width, height);
    }
    if config.vertical {
        flip_vertical(buffer, width, height);
    }
}

fn flip_horizontal(buf: &mut [u8], w: usize, h: usize) {
    for y in 0..h {
        let row_start = y * w * 4;
        for x in 0..(w / 2) {
            let left = row_start + x * 4;
            let right = row_start + (w - 1 - x) * 4;
            for i in 0..4 {
                buf.swap(left + i, right + i);
            }
        }
    }
}

fn flip_vertical(buf: &mut [u8], w: usize, h: usize) {
    for y in 0..(h / 2) {
        let top = y * w * 4;
        let bottom = (h - 1 - y) * w * 4;
        for i in 0..(w * 4) {
            buf.swap(top + i, bottom + i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer(w: usize, h: usize) -> Vec<u8> {
        let mut buf = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) * 4;
                buf[idx] = x as u8;     // R
                buf[idx+1] = y as u8;   // G
            }
        }
        buf
    }

    #[test]
    fn test_flip_horizontal() {
        let mut buf = create_test_buffer(4, 2);
        flip_horizontal(&mut buf, 4, 2);
        // После горизонтального отражения R для (0,0) должен стать равным старому (3,0)
        assert_eq!(buf[0], 3); // x=3 -> R=3
        assert_eq!(buf[4], 2); // x=2 -> R=2
    }

    #[test]
    fn test_flip_vertical() {
        let mut buf = create_test_buffer(2, 4);
        flip_vertical(&mut buf, 2, 4);
        // После вертикального отражения G для (0,0) должен стать равным старому (0,3)
        assert_eq!(buf[1], 3); // y=3 -> G=3
    }
}