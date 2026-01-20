# OCR C API - Simple版本

这是一个简化版本的OCR C API，将模型文件内置到动态库中，提供简洁易用的接口。

## 特性

- **内置模型**: 模型文件打包到动态库中，无需外部模型文件
- **简化API**: 只提供6个核心函数，易于理解和使用
- **自动资源管理**: 自动处理临时文件和资源清理
- **零配置**: 无需复杂的配置，开箱即用

## 包含的模型

- **检测模型**: PP-OCRv5 mobile det
- **识别模型**: PP-OCRv5 mobile rec
- **字符集**: PP-OCRv5 完整字符集

## 快速开始

### 1. 编译动态库

```bash
cargo build --release
```

编译后会生成:
- macOS: `target/release/libocr_capi.dylib`
- Linux: `target/release/libocr_capi.so`
- Windows: `target/release/ocr_capi.dll` + `target/release/ocr_capi.dll.lib` (导入库)

### 2. 编译并运行C示例

```bash
cd examples/c_simple
./build.sh

# 运行示例
./example /path/to/image.jpg
```

## API文档

### 核心函数

```c
#include "ocr_capi_simple.h"

// 1. 初始化OCR引擎 (应用启动时调用一次)
bool ocr_init(void);

// 2. 检测文本区域
OcrDetResult ocr_detect(const uint8_t* rgb_data, uint32_t width, uint32_t height);

// 3. 识别单个文本行
OcrRecResult ocr_recognize(const uint8_t* rgb_data, uint32_t width, uint32_t height);

// 4. 完整OCR识别 (检测+识别，推荐)
OcrResult ocr_process(const uint8_t* rgb_data, uint32_t width, uint32_t height);

// 5. 清理资源
void ocr_det_result_free(OcrDetResult* result);
void ocr_rec_result_free(OcrRecResult* result);
void ocr_result_free(OcrResult* result);

// 6. 清理引擎 (应用退出时调用)
void ocr_cleanup(void);
```

### 数据结构

```c
// 检测框位置 (4个角点坐标)
typedef struct {
    float x1, y1;  // 左上角
    float x2, y2;  // 右上角
    float x3, y3;  // 右下角
    float x4, y4;  // 左下角
} OcrDetBox;

// 检测结果
typedef struct {
    OcrDetBox* boxes;  // 检测框数组
    size_t count;      // 数量
} OcrDetResult;

// 识别结果
typedef struct {
    char* text;        // 识别的文本 (UTF-8)
    float confidence;  // 置信度 (0.0-1.0)
} OcrRecResult;

// 完整OCR结果项
typedef struct {
    OcrDetBox bbox;    // 文本框位置
    char* text;        // 识别的文本
    float confidence;  // 置信度
} OcrResultItem;

// 完整OCR结果
typedef struct {
    OcrResultItem* items;  // 结果数组
    size_t count;          // 数量
} OcrResult;
```

## 使用示例

### 基础用法

```c
#include "ocr_capi_simple.h"
#include <stdio.h>

int main() {
    // 1. 初始化
    if (!ocr_init()) {
        printf("初始化失败\n");
        return 1;
    }
    
    // 2. 准备图像数据 (RGB格式)
    uint8_t* rgb_data = ...; // 加载你的图像
    uint32_t width = ...;
    uint32_t height = ...;
    
    // 3. 执行OCR
    OcrResult result = ocr_process(rgb_data, width, height);
    
    // 4. 处理结果
    for (size_t i = 0; i < result.count; i++) {
        printf("文本: %s\n", result.items[i].text);
        printf("置信度: %.2f%%\n", result.items[i].confidence * 100);
    }
    
    // 5. 清理
    ocr_result_free(&result);
    ocr_cleanup();
    
    return 0;
}
```

### 仅检测文本区域

```c
OcrDetResult det_result = ocr_detect(rgb_data, width, height);

for (size_t i = 0; i < det_result.count; i++) {
    OcrDetBox* box = &det_result.boxes[i];
    printf("文本框: (%.1f,%.1f) -> (%.1f,%.1f)\n",
           box->x1, box->y1, box->x3, box->y3);
}

ocr_det_result_free(&det_result);
```

## 编译选项

### 使用CMake

```cmake
# 链接OCR库
target_link_libraries(your_app
    PRIVATE
    ocr_capi
)

# 设置rpath (Linux/macOS)
set_target_properties(your_app PROPERTIES
    BUILD_RPATH "$ORIGIN"
    INSTALL_RPATH "$ORIGIN"
)
```

### 使用gcc/clang直接编译

```bash
# macOS
gcc -o app app.c \
    -I/path/to/include \
    -L/path/to/target/release \
    -locr_capi \
    -Wl,-rpath,@loader_path

# Linux
gcc -o app app.c \
    -I/path/to/include \
    -L/path/to/target/release \
    -locr_capi \
    -Wl,-rpath,\$ORIGIN

# Windows (使用MSVC)
cl.exe app.c /I\path\to\include /link /LIBPATH:\path\to\target\release ocr_capi.dll.lib

# Windows (使用MinGW)
gcc -o app app.c \
    -I/path/to/include \
    -L/path/to/target/release \
    -locr_capi
```

## 注意事项

1. **图像格式**: 输入必须是RGB格式(每像素3字节)
2. **资源释放**: 所有返回的结果都需要调用对应的free函数释放
3. **初始化**: `ocr_init()` 只需在应用启动时调用一次
4. **线程安全**: 当前实现使用全局锁，支持多线程但性能可能受限

## 错误处理

所有函数失败时都会设置错误信息：

```c
if (!ocr_init()) {
    const char* err = ocr_get_last_error();
    printf("错误: %s\n", err ? err : "未知错误");
}
```

## 完整示例

参见 [examples/c_simple/example.c](examples/c_simple/example.c)

## 开发

### 构建要求

- Rust 1.70+
- C编译器 (gcc/clang)
- MNN库依赖 (自动处理)

### 修改模型

要使用不同的模型，修改 `models/` 目录中的文件并重新编译：

```bash
# 替换模型文件
cp your_det_model.mnn models/PP-OCRv5_mobile_det_fp16.mnn
cp your_rec_model.mnn models/en_PP-OCRv5_mobile_rec_infer.mnn
cp your_charset.txt models/ppocr_keys_en.txt

# 重新编译
cargo build --release
```

## 许可证

MIT License

