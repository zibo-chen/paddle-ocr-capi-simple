use std::path::PathBuf;

fn main() {
    // 将模型文件嵌入到二进制文件中
    let models_dir = PathBuf::from("models");

    // 确保 models 目录存在
    if !models_dir.exists() {
        panic!("models directory not found!");
    }

    // 定义需要的模型文件
    let model_files = vec![
        "PP-OCRv5_mobile_det_fp16.mnn",
        "PP-OCRv5_mobile_rec_fp16.mnn",
        "ppocr_keys_v5.txt",
    ];

    // 检查模型文件是否存在
    for file in &model_files {
        let path = models_dir.join(file);
        if !path.exists() {
            panic!("Model file not found: {}", path.display());
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }

    println!("cargo:rerun-if-changed=build.rs");
}
