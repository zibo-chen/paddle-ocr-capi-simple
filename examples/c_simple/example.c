#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../../include/ocr_capi_simple.h"

// STB Image implementation
#define STB_IMAGE_IMPLEMENTATION
#define STBI_ONLY_JPEG
#define STBI_ONLY_PNG
#define STBI_ONLY_BMP
#include "stb_image.h"

int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        printf("Usage: %s <image_path>\n", argv[0]);
        return 1;
    }

    const char *image_path = argv[1];

    // 初始化OCR引擎
    printf("Initializing OCR engine...\n");
    if (!ocr_init())
    {
        const char *err = ocr_get_last_error();
        printf("Failed to initialize OCR: %s\n", err ? err : "unknown error");
        return 1;
    }
    printf("OCR engine initialized successfully!\n\n");

    // 加载图像
    printf("Loading image: %s\n", image_path);
    int width, height, channels;
    unsigned char *img_data = stbi_load(image_path, &width, &height, &channels, 3);

    if (!img_data)
    {
        printf("Failed to load image\n");
        ocr_cleanup();
        return 1;
    }
    printf("Image loaded: %dx%d, channels=%d\n\n", width, height, channels);

    // 示例1: 完整OCR识别 (推荐使用)
    printf("=== Full OCR Recognition ===\n");
    OcrResult result = ocr_process(img_data, width, height);

    if (result.count == 0)
    {
        const char *err = ocr_get_last_error();
        printf("OCR failed: %s\n", err ? err : "no text detected");
    }
    else
    {
        printf("Found %zu text regions:\n\n", result.count);
        for (size_t i = 0; i < result.count; i++)
        {
            OcrResultItem *item = &result.items[i];
            printf("  [%zu] Text: %s\n", i + 1, item->text);
            printf("       Confidence: %.2f%%\n", item->confidence * 100);
            printf("       Box: (%.1f,%.1f) (%.1f,%.1f) (%.1f,%.1f) (%.1f,%.1f)\n\n",
                   item->bbox.x1, item->bbox.y1,
                   item->bbox.x2, item->bbox.y2,
                   item->bbox.x3, item->bbox.y3,
                   item->bbox.x4, item->bbox.y4);
        }
    }
    ocr_result_free(&result);

    // 示例2: 仅检测文本区域
    printf("\n=== Text Detection Only ===\n");
    OcrDetResult det_result = ocr_detect(img_data, width, height);

    if (det_result.count == 0)
    {
        printf("No text detected\n");
    }
    else
    {
        printf("Detected %zu text regions:\n", det_result.count);
        for (size_t i = 0; i < det_result.count; i++)
        {
            OcrDetBox *box = &det_result.boxes[i];
            printf("  [%zu] Box: (%.1f,%.1f) (%.1f,%.1f) (%.1f,%.1f) (%.1f,%.1f)\n",
                   i + 1, box->x1, box->y1, box->x2, box->y2,
                   box->x3, box->y3, box->x4, box->y4);
        }
    }
    ocr_det_result_free(&det_result);

    // 清理资源
    stbi_image_free(img_data);
    ocr_cleanup();

    printf("\nDone!\n");
    return 0;
}
