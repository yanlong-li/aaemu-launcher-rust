use std::path::Path;

use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::win_main::WmCommand::Progress;
use futures::StreamExt;
use tokio::fs;

pub async fn download(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("开始下载文件 {} ====> {}", url, filename);

    let path = Path::new(&filename);
    let client = Client::new();
    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);


    let mut downloaded: u64 = 0;

    // 获取文件目录
    if let Some(parent) = Path::new(path).parent() {
        // 创建目录（如果不存在）
        println!("目录不存在，自动创建! {:?}", parent);
        fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(path).await?;

    // 异步处理数据流
    let mut stream = response.bytes_stream();
    let s3 = super::SENDER.get().unwrap().clone();

    // 读取流并计算进度
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percentage = downloaded as f64 / total_size as f64 * 100.0;
            println!("已下载: {:.2}%", percentage);
            // let s3 = ss.unwrap().lock();
            let res = s3.lock().unwrap().send((Progress.into_usize(), percentage as isize)).await;
            println!("进度通知结果 {:?}", res);
        } else {
            println!("已下载: {} bytes", downloaded);
        }
    }
    println!("下载完成");
    Ok(())
}