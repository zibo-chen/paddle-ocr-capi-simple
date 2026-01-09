//! Simplified OCR C API
//!
//! 简化的OCR C接口实现，内置模型文件

use image::{DynamicImage, RgbImage};
use libc::{c_char, c_float, c_uint, size_t};
use ocr_rs::{OcrEngine, OcrEngineConfig};
use once_cell::sync::OnceCell;
use std::ffi::CString;
use std::ptr;
use std::slice;
use std::sync::Mutex;

// ============================================================================
// 内置模型数据
// ============================================================================

static DET_MODEL: &[u8] = include_bytes!("../models/PP-OCRv5_mobile_det_fp16.mnn");
static REC_MODEL: &[u8] = include_bytes!("../models/PP-OCRv5_mobile_rec_fp16.mnn");
static CHARSET: &str = include_str!("../models/ppocr_keys_v5.txt");

// ============================================================================
// 全局引擎实例
// ============================================================================

static OCR_ENGINE: OnceCell<Mutex<Option<OcrEngine>>> = OnceCell::new();
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

fn set_last_error(msg: String) {
    if let Ok(mut err) = LAST_ERROR.lock() {
        *err = Some(msg);
    }
}

fn get_last_error_msg() -> Option<String> {
    LAST_ERROR.lock().ok()?.clone()
}

// ============================================================================
// C数据结构
// ============================================================================

/// 检测框位置
#[repr(C)]
pub struct OcrDetBox {
    pub x1: c_float,
    pub y1: c_float,
    pub x2: c_float,
    pub y2: c_float,
    pub x3: c_float,
    pub y3: c_float,
    pub x4: c_float,
    pub y4: c_float,
}

/// 检测结果
#[repr(C)]
pub struct OcrDetResult {
    pub boxes: *mut OcrDetBox,
    pub count: size_t,
}

/// 识别结果
#[repr(C)]
pub struct OcrRecResult {
    pub text: *mut c_char,
    pub confidence: c_float,
}

/// 完整OCR结果项
#[repr(C)]
pub struct OcrResultItem {
    pub bbox: OcrDetBox,
    pub text: *mut c_char,
    pub confidence: c_float,
}

/// 完整OCR结果
#[repr(C)]
pub struct OcrResult {
    pub items: *mut OcrResultItem,
    pub count: size_t,
}

// ============================================================================
// API实现
// ============================================================================

/// 初始化OCR引擎
#[no_mangle]
pub extern "C" fn ocr_init() -> bool {
    // 清除之前的错误
    if let Ok(mut err) = LAST_ERROR.lock() {
        *err = None;
    }

    // 创建临时文件存储模型
    let temp_dir = std::env::temp_dir();
    let det_path = temp_dir.join("ocr_det.mnn");
    let rec_path = temp_dir.join("ocr_rec.mnn");
    let charset_path = temp_dir.join("ocr_keys.txt");

    // 写入模型文件
    if let Err(e) = std::fs::write(&det_path, DET_MODEL) {
        set_last_error(format!("Failed to write det model: {}", e));
        return false;
    }
    if let Err(e) = std::fs::write(&rec_path, REC_MODEL) {
        set_last_error(format!("Failed to write rec model: {}", e));
        return false;
    }
    if let Err(e) = std::fs::write(&charset_path, CHARSET) {
        set_last_error(format!("Failed to write charset: {}", e));
        return false;
    }

    // 创建引擎配置 - 使用更保守的设置
    let config = OcrEngineConfig::new()
        .with_threads(4)
        .with_det_options(
            ocr_rs::DetOptions::new()
                .with_max_side_len(960)
                .with_precision_mode(ocr_rs::DetPrecisionMode::Fast),
        )
        .with_rec_options(
            ocr_rs::RecOptions::new()
                .with_min_score(0.3)
                .with_batch_size(8),
        );

    // 创建引擎
    match OcrEngine::new(
        det_path.to_str().unwrap(),
        rec_path.to_str().unwrap(),
        charset_path.to_str().unwrap(),
        Some(config),
    ) {
        Ok(engine) => {
            OCR_ENGINE.get_or_init(|| Mutex::new(Some(engine)));
            true
        }
        Err(e) => {
            set_last_error(format!("Failed to create OCR engine: {}", e));
            false
        }
    }
}

/// 检测图像中的文本区域
#[no_mangle]
pub extern "C" fn ocr_detect(rgb_data: *const u8, width: c_uint, height: c_uint) -> OcrDetResult {
    if rgb_data.is_null() {
        set_last_error("rgb_data is null".to_string());
        return OcrDetResult {
            boxes: ptr::null_mut(),
            count: 0,
        };
    }

    let engine = match OCR_ENGINE.get() {
        Some(e) => e,
        None => {
            set_last_error("OCR engine not initialized. Call ocr_init() first".to_string());
            return OcrDetResult {
                boxes: ptr::null_mut(),
                count: 0,
            };
        }
    };

    let mut engine_guard = match engine.lock() {
        Ok(g) => g,
        Err(e) => {
            set_last_error(format!("Failed to lock engine: {}", e));
            return OcrDetResult {
                boxes: ptr::null_mut(),
                count: 0,
            };
        }
    };

    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => {
            set_last_error("Engine is None".to_string());
            return OcrDetResult {
                boxes: ptr::null_mut(),
                count: 0,
            };
        }
    };

    // 将RGB数据转换为图像
    let data_len = (width as usize) * (height as usize) * 3;
    let rgb_slice = unsafe { slice::from_raw_parts(rgb_data, data_len) };

    let img = match RgbImage::from_raw(width, height, rgb_slice.to_vec()) {
        Some(img) => DynamicImage::ImageRgb8(img),
        None => {
            set_last_error("Failed to create image from RGB data".to_string());
            return OcrDetResult {
                boxes: ptr::null_mut(),
                count: 0,
            };
        }
    };

    // 执行检测
    match engine.detect(&img) {
        Ok(det_result) => {
            let count = det_result.len();
            if count == 0 {
                return OcrDetResult {
                    boxes: ptr::null_mut(),
                    count: 0,
                };
            }

            let boxes: Vec<OcrDetBox> = det_result
                .iter()
                .map(|b| {
                    // 使用rect坐标，因为points可能为None
                    let rect = &b.rect;
                    let x1 = rect.left() as f32;
                    let y1 = rect.top() as f32;
                    let x2 = (rect.left() + rect.width() as i32) as f32;
                    let y2 = rect.top() as f32;
                    let x3 = (rect.left() + rect.width() as i32) as f32;
                    let y3 = (rect.top() + rect.height() as i32) as f32;
                    let x4 = rect.left() as f32;
                    let y4 = (rect.top() + rect.height() as i32) as f32;

                    OcrDetBox {
                        x1,
                        y1,
                        x2,
                        y2,
                        x3,
                        y3,
                        x4,
                        y4,
                    }
                })
                .collect();

            let mut boxed_boxes = boxes.into_boxed_slice();
            let ptr = boxed_boxes.as_mut_ptr();
            std::mem::forget(boxed_boxes);

            OcrDetResult { boxes: ptr, count }
        }
        Err(e) => {
            set_last_error(format!("Detection failed: {}", e));
            OcrDetResult {
                boxes: ptr::null_mut(),
                count: 0,
            }
        }
    }
}

/// 识别单个文本行
#[no_mangle]
pub extern "C" fn ocr_recognize(
    rgb_data: *const u8,
    width: c_uint,
    height: c_uint,
) -> OcrRecResult {
    if rgb_data.is_null() {
        set_last_error("rgb_data is null".to_string());
        return OcrRecResult {
            text: ptr::null_mut(),
            confidence: 0.0,
        };
    }

    let engine = match OCR_ENGINE.get() {
        Some(e) => e,
        None => {
            set_last_error("OCR engine not initialized. Call ocr_init() first".to_string());
            return OcrRecResult {
                text: ptr::null_mut(),
                confidence: 0.0,
            };
        }
    };

    let mut engine_guard = match engine.lock() {
        Ok(g) => g,
        Err(e) => {
            set_last_error(format!("Failed to lock engine: {}", e));
            return OcrRecResult {
                text: ptr::null_mut(),
                confidence: 0.0,
            };
        }
    };

    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => {
            set_last_error("Engine is None".to_string());
            return OcrRecResult {
                text: ptr::null_mut(),
                confidence: 0.0,
            };
        }
    };

    // 将RGB数据转换为图像
    let data_len = (width as usize) * (height as usize) * 3;
    let rgb_slice = unsafe { slice::from_raw_parts(rgb_data, data_len) };

    let img = match RgbImage::from_raw(width, height, rgb_slice.to_vec()) {
        Some(img) => DynamicImage::ImageRgb8(img),
        None => {
            set_last_error("Failed to create image from RGB data".to_string());
            return OcrRecResult {
                text: ptr::null_mut(),
                confidence: 0.0,
            };
        }
    };

    // 执行识别 - 这里需要先检测再识别单个文本行
    // 由于API不支持直接识别单个行，我们使用完整流程
    match engine.recognize(&img) {
        Ok(results) => {
            if results.is_empty() {
                set_last_error("No text recognized".to_string());
                return OcrRecResult {
                    text: ptr::null_mut(),
                    confidence: 0.0,
                };
            }

            // 返回第一个结果
            let result = &results[0];
            let text = match CString::new(result.text.as_str()) {
                Ok(s) => s.into_raw(),
                Err(_) => {
                    set_last_error("Failed to convert text to C string".to_string());
                    ptr::null_mut()
                }
            };

            OcrRecResult {
                text,
                confidence: result.confidence,
            }
        }
        Err(e) => {
            set_last_error(format!("Recognition failed: {}", e));
            OcrRecResult {
                text: ptr::null_mut(),
                confidence: 0.0,
            }
        }
    }
}

/// 完整OCR识别
#[no_mangle]
pub extern "C" fn ocr_process(rgb_data: *const u8, width: c_uint, height: c_uint) -> OcrResult {
    if rgb_data.is_null() {
        set_last_error("rgb_data is null".to_string());
        return OcrResult {
            items: ptr::null_mut(),
            count: 0,
        };
    }

    let engine = match OCR_ENGINE.get() {
        Some(e) => e,
        None => {
            set_last_error("OCR engine not initialized. Call ocr_init() first".to_string());
            return OcrResult {
                items: ptr::null_mut(),
                count: 0,
            };
        }
    };

    let mut engine_guard = match engine.lock() {
        Ok(g) => g,
        Err(e) => {
            set_last_error(format!("Failed to lock engine: {}", e));
            return OcrResult {
                items: ptr::null_mut(),
                count: 0,
            };
        }
    };

    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => {
            set_last_error("Engine is None".to_string());
            return OcrResult {
                items: ptr::null_mut(),
                count: 0,
            };
        }
    };

    // 将RGB数据转换为图像
    let data_len = (width as usize) * (height as usize) * 3;
    let rgb_slice = unsafe { slice::from_raw_parts(rgb_data, data_len) };

    let img = match RgbImage::from_raw(width, height, rgb_slice.to_vec()) {
        Some(img) => DynamicImage::ImageRgb8(img),
        None => {
            set_last_error("Failed to create image from RGB data".to_string());
            return OcrResult {
                items: ptr::null_mut(),
                count: 0,
            };
        }
    };

    // 执行完整OCR
    match engine.recognize(&img) {
        Ok(results) => {
            let count = results.len();
            if count == 0 {
                return OcrResult {
                    items: ptr::null_mut(),
                    count: 0,
                };
            }

            let items: Vec<OcrResultItem> = results
                .iter()
                .map(|r| {
                    let text = CString::new(r.text.as_str())
                        .unwrap_or_else(|_| CString::new("").unwrap())
                        .into_raw();

                    // 使用rect坐标而不是points
                    let rect = &r.bbox.rect;
                    let x1 = rect.left() as f32;
                    let y1 = rect.top() as f32;
                    let x2 = (rect.left() + rect.width() as i32) as f32;
                    let y2 = rect.top() as f32;
                    let x3 = (rect.left() + rect.width() as i32) as f32;
                    let y3 = (rect.top() + rect.height() as i32) as f32;
                    let x4 = rect.left() as f32;
                    let y4 = (rect.top() + rect.height() as i32) as f32;

                    OcrResultItem {
                        bbox: OcrDetBox {
                            x1,
                            y1,
                            x2,
                            y2,
                            x3,
                            y3,
                            x4,
                            y4,
                        },
                        text,
                        confidence: r.confidence,
                    }
                })
                .collect();

            let mut boxed_items = items.into_boxed_slice();
            let ptr = boxed_items.as_mut_ptr();
            std::mem::forget(boxed_items);

            OcrResult { items: ptr, count }
        }
        Err(e) => {
            set_last_error(format!("OCR failed: {}", e));
            OcrResult {
                items: ptr::null_mut(),
                count: 0,
            }
        }
    }
}

/// 释放检测结果
#[no_mangle]
pub extern "C" fn ocr_det_result_free(result: *mut OcrDetResult) {
    if result.is_null() {
        return;
    }

    unsafe {
        let result = &mut *result;
        if !result.boxes.is_null() && result.count > 0 {
            let _ = Box::from_raw(
                slice::from_raw_parts_mut(result.boxes, result.count) as *mut [OcrDetBox]
            );
            result.boxes = ptr::null_mut();
            result.count = 0;
        }
    }
}

/// 释放识别结果
#[no_mangle]
pub extern "C" fn ocr_rec_result_free(result: *mut OcrRecResult) {
    if result.is_null() {
        return;
    }

    unsafe {
        let result = &mut *result;
        if !result.text.is_null() {
            let _ = CString::from_raw(result.text);
            result.text = ptr::null_mut();
        }
    }
}

/// 释放完整OCR结果
#[no_mangle]
pub extern "C" fn ocr_result_free(result: *mut OcrResult) {
    if result.is_null() {
        return;
    }

    unsafe {
        let result = &mut *result;
        if !result.items.is_null() && result.count > 0 {
            let items_slice = slice::from_raw_parts_mut(result.items, result.count);
            for item in items_slice.iter_mut() {
                if !item.text.is_null() {
                    let _ = CString::from_raw(item.text);
                }
            }
            let _ = Box::from_raw(items_slice as *mut [OcrResultItem]);
            result.items = ptr::null_mut();
            result.count = 0;
        }
    }
}

/// 清理OCR引擎资源
#[no_mangle]
pub extern "C" fn ocr_cleanup() {
    if let Some(engine) = OCR_ENGINE.get() {
        if let Ok(mut guard) = engine.lock() {
            *guard = None;
        }
    }

    // 清理临时模型文件
    let temp_dir = std::env::temp_dir();
    let _ = std::fs::remove_file(temp_dir.join("ocr_det.mnn"));
    let _ = std::fs::remove_file(temp_dir.join("ocr_rec.mnn"));
    let _ = std::fs::remove_file(temp_dir.join("ocr_keys.txt"));
}

/// 获取最后一次错误信息
#[no_mangle]
pub extern "C" fn ocr_get_last_error() -> *const c_char {
    use std::sync::OnceLock;
    static ERROR_BUFFER: OnceLock<Mutex<Option<CString>>> = OnceLock::new();

    if let Some(msg) = get_last_error_msg() {
        let buffer = ERROR_BUFFER.get_or_init(|| Mutex::new(None));
        if let Ok(mut guard) = buffer.lock() {
            *guard = CString::new(msg).ok();
            return guard.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());
        }
    }
    ptr::null()
}
