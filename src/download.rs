use std::time::Duration;

use crate::win_main::WmCommand::Progress;

pub async fn download(url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    for i in 0..101 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("当前下载进度 {}%", i);
        let ss = super::SENDER.get();
        let s2 = ss.clone();

        let s3 = s2.unwrap().lock();

        let res  = s3.unwrap().send((Progress.into_usize(), i)).await;
        println!("进度通知结果 {:?}",res);
    }

    // let path = Path::new(&filename);
    // let client = Client::new();
    //
    // let total_size = {
    //     let resp = client.head(url).send().await?;
    //     if resp.status().is_success() {
    //         resp.headers()
    //             .get(header::CONTENT_LENGTH)
    //             .and_then(|ct_len| ct_len.to_str().ok())
    //             .and_then(|ct_len| ct_len.parse().ok())
    //             .unwrap_or(0)
    //     } else {
    //         return Err(Box::from(anyhow!(
    //             "Couldn't download URL: {}. Error: {:?}",
    //             url,
    //             resp.status(),
    //         )));
    //     }
    // };
    //
    // let mut request = client.get(url);
    // let pb = ProgressBar::new(total_size);
    // pb.set_style(ProgressStyle::default_bar()
    //     .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
    //     .progress_chars("#>-"));
    //
    // if path.exists() {
    //     let size = path.metadata()?.len().saturating_sub(1);
    //     request = request.header(header::RANGE, format!("bytes={}-", size));
    //     pb.inc(size);
    // }
    // let mut source = request.send().await?;
    // let mut dest = fs::OpenOptions::new().create(true).append(true).open(&path)?;
    // while let Some(chunk) = source.chunk().await? {
    //     dest.write_all(&chunk)?;
    //     pb.inc(chunk.len() as u64);
    // }
    // println!("下载完成");
    Ok(())
}