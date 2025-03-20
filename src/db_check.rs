use std::env;
use std::fs::File;
use std::path::Path;

use chksum_md5 as md5;
// use reqwest::blocking::Client;
// use windows::core::w;
// use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW};

const DB_PATH: &str = "/game/db/compact.sqlite3";


pub fn detect_db(hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();
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

// pub fn handle(hash: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();
//
//
//     let exe_path = format!("{}{}", root_path, DB_PATH);
//
//     if !Path::exists(exe_path.as_ref()) {
//         unsafe { MessageBoxW(None, w!("游戏资源丢失，需要更新！"), w!("游戏资源有更新"), MB_OK); }
//
//         // if start_download_db(exe_path.as_ref()).is_err() {
//         //     unsafe { MessageBoxW(None, w!("更新游戏资源失败，请联系QQ群。"), w!("游戏资源有更新"), MB_OK); }
//         //     return Err(Box::from("游戏资源更新失败"));
//         // }
//     }
//
//     let file = File::open(&exe_path)?;
//     let digest = md5::chksum(file)?;
//
//
//     if digest.to_hex_lowercase() != hash {
//         unsafe { MessageBoxW(None, w!("游戏资源有新版本需要更新！"), w!("游戏资源有更新"), MB_OK); }
//         // if (start_download_db(exe_path.as_ref()).is_err()) {
//         //     unsafe { MessageBoxW(None, w!("更新游戏资源失败，请联系QQ群。"), w!("游戏资源有更新"), MB_OK); }
//         //     return Err(Box::from("游戏资源更新失败"));
//         // }
//     }
//
//     Ok(())
// }


// pub async  fn download_file(url: &str, save_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
//     // 创建一个 HTTP 客户端
//     let client = Client::new();
//     // 发送 GET 请求到指定的 URL
//     let mut response = client.get(url).send()?;
//
//     // 检查响应状态
//     if !response.status().is_success() {
//         return Err(format!("Failed to download file: {}", response.status()).into());
//     }
//
//     // 创建本地文件
//     let mut file = File::create(save_path)?;
//     // 将响应内容复制到文件中
//     copy(&mut response, &mut file)?;
//
//     println!("File downloaded to {:?}", save_path);
//     Ok(())
// }

// pub async  fn start_download_db() -> Result<(), Box<dyn std::error::Error>> {
//     let url = "https://plaa.top/compact.sqlite3";
//
//     let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();
//
//     let exe_path = format!("{}{}", root_path, DB_PATH);
//
//     download_file(url, Path::new(&exe_path)).await
// }
