// #![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::mpsc::{channel, Sender};

mod trion_1_2;
mod web_site;

mod regedit;

mod cipher;
mod protocol;

mod helper;

mod business_logic;
mod db_check;
mod download;
mod system_config;
mod win32api;
mod win_main;

mod resources;

mod site_link_url;

mod task;

const WEBSITE_URL: &str = "https://plaa.top";

const VERSION: u16 = 3;

static SENDER: OnceLock<Arc<Mutex<Sender<(usize, isize)>>>> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // initialize tracing
    tracing_subscriber::fmt::init();

    resources::release();

    let (tx, mut rx) = channel::<(usize, isize)>(10);
    SENDER.set(Arc::new(Mutex::new(tx.clone()))).unwrap();

    tracing::info!("程序启动...");
    let hwnd = win_main::handle().await;

    println!("任务1");
    tokio::join!(business_logic::handle(hwnd));

    let task1 = task::task(&mut rx, hwnd);

    let task2 = win32api::handle_msg();

    tokio::select! {
      _ = task1=>{},
      _ = task2=>{},
    };

    println!("任务2");
    Ok(())
}
