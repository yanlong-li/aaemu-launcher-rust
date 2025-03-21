use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 资源目录
const RESOURCES_DIR: &str = "./resources";

/// 文件列表：文件名 -> 读取数据
const FILES: &[(&str, &[u8])] = &[
    (
        "background.bmp",
        include_bytes!("../resources/background.bmp"),
    ),
    ("exit.bmp", include_bytes!("../resources/exit.bmp")),
    (
        "play_button_active.bmp",
        include_bytes!("../resources/play_button_active.bmp"),
    ),
    ("play.bmp", include_bytes!("../resources/play.bmp")),
    ("upgrade.bmp", include_bytes!("../resources/upgrade.bmp")),
];

pub fn release() {
    let path = Path::new(RESOURCES_DIR);

    // 确保资源目录存在
    if !path.exists() {
        fs::create_dir_all(path).unwrap();
    }

    // 遍历文件列表，检查并写入文件
    for (file_name, data) in FILES.iter() {
        let file_path: PathBuf = path.join(file_name); // 拼接完整路径

        if !file_path.exists() {
            File::create(&file_path).unwrap().write_all(data).unwrap();
        }
    }
}
