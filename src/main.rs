// #![windows_subsystem = "windows"]

use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::sleep;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, SetTimer, WM_COMMAND};

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

const WEBSITE_URL: &str = "https://plaa.top";

const VERSION: u16 = 2;

async fn set_progress(hwnd: HWND, step: usize) {
    println!("设置进度");
    unsafe {
        SendMessageW(hwnd, WM_COMMAND, WPARAM(step), LPARAM(0));
    }
}


static SENDER: OnceLock<Arc<Mutex<Sender<(usize, isize)>>>> = OnceLock::new();
static RECEIVER: OnceLock<Receiver<(usize, isize)>> = OnceLock::new();
static mut MAIN_HWND: OnceLock<HWND> = OnceLock::new();


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = channel::<(usize, isize)>(1);
    SENDER.set(Arc::new(Mutex::new(tx.clone()))).unwrap();
    // RECEIVER.set(&mut rx).unwrap();

    // let s = tx.clone();

    // tokio::task::spawn(async move {
    //     // let mut e = a;
    //     loop {
    //         s.send((0, 0)).await;
    //         println!("send");
    //         sleep(Duration::from_secs(1)).await;
    //     }
    // });
    //


    println!("程序启动...");
    let hwnd = win_main::handle().await;
    // 设置一个定时器，定时触发下消息事件
    unsafe {
        SetTimer(hwnd, 1, 10, None);
        MAIN_HWND = OnceLock::from(hwnd);
    }

    // tokio::task::spawn(async move {
    //     loop {
    //         let v = rx.recv().await.unwrap();
    //
    //         println!("recv {:?}", v);
    //     }
    // });

    win32api::handle_msg();
    Ok(())
}
