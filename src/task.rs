use crate::business_logic::handle_launch;
use crate::win_main::WmCommand::{PlayButton, Progress, StartUpgrade};
use crate::{download, protocol};
use std::env;
use tokio::sync::mpsc::Receiver;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_COMMAND};

pub async fn task(rx: &mut Receiver<(usize, isize)>, hwnd: HWND) {
    println!("等待接收任务");
    loop {
        let s = rx.recv().await;
        println!("任务已接受 {:?}", s);
        match s {
            None => {}
            Some(res) => match res.0 {
                val if val == Progress.into_usize() => {
                    println!("收到 Progress");
                    unsafe {
                        SendMessageW(
                            hwnd,
                            WM_COMMAND,
                            WPARAM(Progress.into_usize()),
                            LPARAM(res.1),
                        );
                    };
                }
                val if val == StartUpgrade.into_usize() => {
                    let auth_token = protocol::handle().await.unwrap();
                    let root_path = env::current_exe()
                        .expect("获取当前路径失败")
                        .parent()
                        .expect("获取父级目录")
                        .to_str()
                        .expect("转换为字符串失败")
                        .to_string();
                    let db_path = format!("{}{}", root_path, "/game/db/compact.sqlite3");
                    let down_res = download::download(
                        &format!("{}/compact.sqlite3", auth_token.domain),
                        &db_path,
                    )
                    .await;
                    println!("文件下载结果 {:?}", down_res);
                }
                val if val == PlayButton.into_usize() => {
                    let auth_token = protocol::handle().await.unwrap();
                    handle_launch(&auth_token).await;
                }
                _ => {
                    println!("未知事件 {:?}", res.0);
                }
            },
        };
    }
}
