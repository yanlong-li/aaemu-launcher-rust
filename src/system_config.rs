use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{info, warn};

pub async fn update() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"-- [SYSTEM CONFIGURATION]
-- WARNING! THIS FILE IS GENERATED BY THE SYSTEM! EDITING IS NOT RECOMMENDED!
sys_spec_full = 4
r_driver = "DX10"
r_multithreaded = 1
option_sound = 4
r_fullscreen = 0
r_vsync = 0
locale = zh_cn
    "#;

    let homepath = std::env::var("USERPROFILE").or_else(|_| {
        std::env::var("HOMEPATH")
    }).expect("获取用户环境失败 HOMEPATH");


    let file_name = "system.cfg";

    // 创建一个包含目录和文件名的完整路径
    let file_path = format!("{}\\Documents\\AAEmu\\{}", homepath, file_name);
    let file_path = Path::new(&file_path);

    // 检查文件是否存在
    if !file_path.exists() {
        info!("File does not exist: {:?}", file_path);

        // 检查目录是否存在，如果不存在则创建目录
        if let Some(parent_dir) = file_path.parent() {
            if !parent_dir.exists() {
                info!("Directory does not exist: {:?}", parent_dir);
                fs::create_dir_all(parent_dir)?; // 创建所有不存在的父目录
            }
        }

        // 创建文件并写入指定内容
        let mut file = File::create(file_path)?;
        file.write_all(content.as_bytes())?; // 将内容写入文件
        info!("File created: {:?}", file_path);
    } else {
        warn!("File already exists: {:?}", file_path);
    }

    Ok(())
}