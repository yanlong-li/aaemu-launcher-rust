use std::env;
use std::fs::File;
use std::path::Path;

use chksum_md5 as md5;

const DB_PATH: &str = "/game/db/compact.sqlite3";

pub fn detect_db(hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = env::current_exe()
        .expect("获取当前路径失败")
        .parent()
        .expect("获取父级目录")
        .to_str()
        .expect("转换为字符串失败")
        .to_string();
    let exe_path = format!("{}{}", root_path, DB_PATH);

    if !Path::exists(exe_path.as_ref()) {
        return Err("文件不存在".into());
    }

    let file = File::open(&exe_path)?;
    let digest = md5::chksum(file)?;

    if digest.to_hex_lowercase() != hash {
        return Err("文件已变更".into());
    }

    Ok(())
}


