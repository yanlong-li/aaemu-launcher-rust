#![windows_subsystem = "windows"]

use std::env;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, SetTimer, WM_COMMAND};
use crate::business_logic::handle_launch;
use crate::win_main::WmCommand::{PlayButton, Progress, StartUpgrade};

mod trion_1_2;
mod web_site;

mod regedit;

mod protocol;
mod cipher;

mod helper;

mod system_config;
mod db_check;
mod win_main;
mod win32api;
mod business_logic;
mod download;

mod site_link_url;

const WEBSITE_URL: &str = "https://plaa.top";

const VERSION: u16 = 3;

async fn set_progress(hwnd: HWND, step: usize) {
    println!("设置进度");
    unsafe {
        SendMessageW(hwnd, WM_COMMAND, WPARAM(step), LPARAM(0));
    }
}


static SENDER: OnceLock<Arc<Mutex<Sender<(usize, isize)>>>> = OnceLock::new();
static RECEIVER: OnceLock<Receiver<(usize, isize)>> = OnceLock::new();
static mut MAIN_HWND: OnceLock<HWND> = OnceLock::new();

type AsyncTask = Pin<Box<dyn Future<Output=()> + Send>>;

enum TaskType {
    Download,
    Play,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = channel::<(usize, isize)>(10);
    SENDER.set(Arc::new(Mutex::new(tx.clone()))).unwrap();

    println!("程序启动...");
    let hwnd = win_main::handle().await;
    // 设置一个定时器，定时触发下消息事件
    unsafe {
        SetTimer(hwnd, 1, 10, None);
        MAIN_HWND = OnceLock::from(hwnd);
    }

    tokio::join!( business_logic::handle(hwnd));
    println!("任务1");

    let (task_tx, mut task_rx) = channel(50);

    let task2 = async {
        println!("等待接收");

        let task_sender = task_tx.clone();
        loop {
            let s = rx.recv().await;
            println!("已接受 {:?}", s);
            match s {
                None => {}
                Some(res) => {
                    println!("{:?}", res);
                    match res.0 {
                        val if val == Progress.into_usize() => {
                            println!("收到 Progress");
                            unsafe {
                                SendMessageW(hwnd, WM_COMMAND, WPARAM(Progress.into_usize()), LPARAM(res.1));
                            };
                        }
                        val if val == StartUpgrade.into_usize() => {
                            println!("开始下载任务");
                            task_sender.send(TaskType::Download).await;
                        }
                        val if val == PlayButton.into_usize() => {
                            println!("开始游戏任务");
                            task_sender.send(TaskType::Play).await;
                        }
                        _ => {
                            println!("未知事件 {:?}", res.0);
                        }
                    }
                }
            };
        }
    };

    let task3 = win32api::handle_msg();


    let task4 = async {
        println!("等待接收任务");
        while let task = task_rx.recv().await.unwrap() {
            println!("开始执行任务...");
            match task {
                TaskType::Download => {
                    let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();
                    let db_path = format!("{}{}", root_path, "/game/db/compact.sqlite3");
                    let down_res = download::download("https://plaa.top/compact.sqlite3", &db_path).await;
                    println!("文件下载结果 {:?}", down_res);
                }
                TaskType::Play => {
                    let auth_token = protocol::handle().await.unwrap();
                    handle_launch(&auth_token).await;
                }
            }
        }
    };

    tokio::select! {
      _ = task2=>{},
      _ = task3=>{},
      _ = task4=>{},
    }
    ;

    println!("任务2");
    Ok(())
}
