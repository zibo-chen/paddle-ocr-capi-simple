# Simplified OCR C API Example

这是一个简化的OCR C API示例，展示了如何使用内置模型的OCR功能。

## 特性

- ✅ 模型内置到动态库中，无需外部模型文件
- ✅ 简单的API设计，只需6个函数
- ✅ 支持检测、识别和完整OCR流程
- ✅ 自动资源管理

## API概览

```c
// 1. 初始化 (应用启动时调用一次)
bool ocr_init(void);

// 2. 文本检测
OcrDetResult ocr_detect(const uint8_t* rgb_data, uint32_t width, uint32_t height);

// 3. 文本识别
OcrRecResult ocr_recognize(const uint8_t* rgb_data, uint32_t width, uint32_t height);

// 4. 完整OCR (检测+识别，推荐使用)
OcrResult ocr_process(const uint8_t* rgb_data, uint32_t width, uint32_t height);

// 5. 释放资源
void ocr_det_result_free(OcrDetResult* result);
void ocr_rec_result_free(OcrRecResult* result);
void ocr_result_free(OcrResult* result);

// 6. 清理 (应用退出时调用)
void ocr_cleanup(void);
```

## 编译和运行

### 1. 编译库和示例

```bash
./build.sh
```

### 2. 运行示例

```bash
./example /path/to/image.jpg
```

## 快速开始

```c
#include "ocr_capi_simple.h"

int main() {
    // 初始化
    if (!ocr_init()) {
        printf("Failed to init\n");
        return 1;
    }
    
    // 加载图像为RGB数据
    // uint8_t* rgb_data = ...;
    // uint32_t width = ...;
    // uint32_t height = ...;
    
    // 执行OCR
    OcrResult result = ocr_process(rgb_data, width, height);
    
    // 处理结果
    for (size_t i = 0; i < result.count; i++) {
        printf("Text: %s (%.2f%%)\n", 
               result.items[i].text, 
               result.items[i].confidence * 100);
    }
    
    // 清理
    ocr_result_free(&result);
    ocr_cleanup();
    
    return 0;
}
```

## 错误处理

所有函数失败时都会设置错误信息，可通过 `ocr_get_last_error()` 获取：

```c
if (!ocr_init()) {
    const char* err = ocr_get_last_error();
    printf("Error: %s\n", err ? err : "unknown");
}
```

## 内置模型

- 检测模型: PP-OCRv5 mobile det (FP16)
- 识别模型: PP-OCRv5 mobile rec (English)
- 字符集: 英文字符集

模型会在首次调用 `ocr_init()` 时自动提取到临时目录。

## 注意事项

1. 输入图像必须是RGB格式 (每像素3字节)
2. 所有返回的字符串和数组都需要调用对应的free函数释放
3. `ocr_init()` 只需在应用启动时调用一次
4. `ocr_cleanup()` 应在应用退出时调用

## 性能提示

- 首次调用会稍慢(需要加载模型)
- 后续调用性能较好
- 可考虑使用线程池处理多张图片
