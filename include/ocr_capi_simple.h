/**
 * @file ocr_capi.h
 * @brief Simplified OCR C API
 *
 * 简化的OCR C接口，内置模型文件
 */

#ifndef OCR_CAPI_H
#define OCR_CAPI_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C"
{
#endif

    /* ============================================================================
     * 数据结构
     * ============================================================================ */

    /**
     * 检测框位置
     */
    typedef struct
    {
        float x1, y1; /**< 左上角坐标 */
        float x2, y2; /**< 右上角坐标 */
        float x3, y3; /**< 右下角坐标 */
        float x4, y4; /**< 左下角坐标 */
    } OcrDetBox;

    /**
     * 检测结果
     */
    typedef struct
    {
        OcrDetBox *boxes; /**< 检测框数组 */
        size_t count;     /**< 检测框数量 */
    } OcrDetResult;

    /**
     * 识别结果
     */
    typedef struct
    {
        char *text;       /**< 识别的文本 (UTF-8) */
        float confidence; /**< 置信度 (0.0-1.0) */
    } OcrRecResult;

    /**
     * 完整OCR结果项
     */
    typedef struct
    {
        OcrDetBox bbox;   /**< 文本框位置 */
        char *text;       /**< 识别的文本 (UTF-8) */
        float confidence; /**< 置信度 (0.0-1.0) */
    } OcrResultItem;

    /**
     * 完整OCR结果
     */
    typedef struct
    {
        OcrResultItem *items; /**< 结果数组 */
        size_t count;         /**< 结果数量 */
    } OcrResult;

    /* ============================================================================
     * API 函数
     * ============================================================================ */

    /**
     * 初始化OCR引擎 (使用内置模型)
     *
     * @return true=成功, false=失败
     *
     * @note 此函数会加载内置的英文模型，应用启动时调用一次
     */
    bool ocr_init(void);

    /**
     * 检测图像中的文本区域
     *
     * @param rgb_data RGB图像数据 (格式: RGBRGBRGB...)
     * @param width 图像宽度
     * @param height 图像高度
     * @return 检测结果，使用完毕后需调用 ocr_det_result_free 释放
     */
    OcrDetResult ocr_detect(const uint8_t *rgb_data, uint32_t width, uint32_t height);

    /**
     * 识别单个文本行
     *
     * @param rgb_data RGB图像数据 (裁剪后的文本行图像)
     * @param width 图像宽度
     * @param height 图像高度
     * @return 识别结果，使用完毕后需调用 ocr_rec_result_free 释放
     */
    OcrRecResult ocr_recognize(const uint8_t *rgb_data, uint32_t width, uint32_t height);

    /**
     * 完整OCR识别 (检测 + 识别)
     *
     * @param rgb_data RGB图像数据
     * @param width 图像宽度
     * @param height 图像高度
     * @return OCR结果，使用完毕后需调用 ocr_result_free 释放
     */
    OcrResult ocr_process(const uint8_t *rgb_data, uint32_t width, uint32_t height);

    /**
     * 释放检测结果
     * @param result 检测结果指针
     */
    void ocr_det_result_free(OcrDetResult *result);

    /**
     * 释放识别结果
     * @param result 识别结果指针
     */
    void ocr_rec_result_free(OcrRecResult *result);

    /**
     * 释放完整OCR结果
     * @param result OCR结果指针
     */
    void ocr_result_free(OcrResult *result);

    /**
     * 清理OCR引擎资源
     *
     * @note 应用退出时调用
     */
    void ocr_cleanup(void);

    /**
     * 获取最后一次错误信息
     * @return 错误信息字符串，无错误时返回 NULL
     */
    const char *ocr_get_last_error(void);

#ifdef __cplusplus
}
#endif

#endif // OCR_CAPI_H
