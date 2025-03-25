use crate::Task::Progress;
use crate::{download, protocol, PAUSE_UPGRADE};
use futures::StreamExt;
use reqwest::Client;
use std::env;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

pub async fn download(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    debug!("开始下载文件 {} ====> {}", url, filename);

    let path = Path::new(&filename);
    let client = Client::new();
    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    info!("文件大小 {} bytes", total_size);

    let mut downloaded: u64 = 0;

    // 获取文件目录
    if let Some(parent) = Path::new(path).parent() {
        // 创建目录（如果不存在）
        info!("目录不存在，自动创建! {:?}", parent);
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;

    // 异步处理数据流
    let mut stream = response.bytes_stream();
    let s3 = super::SENDER.get().unwrap().clone();

    // 读取流并计算进度
    while let Some(chunk) = stream.next().await {
        if let Some(pause) = PAUSE_UPGRADE.get() {
            while pause.load(std::sync::atomic::Ordering::Relaxed) {
                debug!("下载任务已暂停");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percentage = downloaded as f64 / total_size as f64 * 100.0;
            info!("已下载: {:.2}% {} {}", percentage, downloaded, total_size);
            let res = s3.lock().await.send(Progress(percentage)).await;
            debug!("进度通知结果 {:?}", res);
        } else {
            info!("已下载: {} bytes", downloaded);
        }
    }
    let _res = s3.lock().await.send(Progress(100f64)).await;
    info!("下载完成");
    Ok(())
}

pub async fn start_download_db() -> Result<(), Box<dyn std::error::Error>> {
    let auth_token = protocol::handle().await.unwrap();
    let root_path = env::current_exe()
        .expect("获取当前路径失败")
        .parent()
        .expect("获取父级目录")
        .to_str()
        .expect("转换为字符串失败")
        .to_string();
    let db_path = format!("{}{}", root_path, "/game/db/compact.sqlite3");

    download::download(&format!("{}/compact.sqlite3", auth_token.domain), &db_path).await
}
