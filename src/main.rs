#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

slint::include_modules!();

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

mod site_link_url;

mod task;

const WEBSITE_URL: &str = "https://plaa.top";

const VERSION: u16 = 3;

static SENDER: OnceLock<Arc<Mutex<Sender<Task>>>> = OnceLock::new();
static PAUSE_UPGRADE: OnceLock<Arc<AtomicBool>> = OnceLock::new();

// static APP_MAIN_WINDOW: OnceLock<MainWindow> = OnceLock::new();

#[derive(Debug)]
enum Task {
    Progress(f64),
    Message(String, String, MessageActions),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    debug!("这是 debug 级别的日志");
    info!("这是 info 级别的日志");
    warn!("这是 warn 级别的日志");
    error!("这是 error 级别的日志");

    let (tx, mut rx) = channel::<Task>(1);
    SENDER.set(Arc::new(Mutex::new(tx.clone()))).unwrap();

    info!("程序启动...");

    let main_window = MainWindow::new()?;

    main_window.on_exit(|| {
        std::process::exit(0);
    });

    main_window.on_start_game(|| {
        eprintln!("开始游戏");

        slint::spawn_local(async move {
            let auth_token = protocol::handle().await.unwrap();
            business_logic::handle_launch(&auth_token).await;
        })
        .expect("TODO: panic message");
    });
    main_window.on_open_website(|| {
        web_site::open_website(WEBSITE_URL).expect("TODO: panic message");
    });
    main_window.on_upgrade(|| {
        debug!("开始更新DB");
        slint::spawn_local(async_compat::Compat::new(async move {
            debug!("开始更新DB2");

            download::start_download_db().await.unwrap();
        }))
        .expect("TODO: panic message");
    });
    // let pause_flag = Arc::new(AtomicBool::new(true)); // 初始状态：允许下载

    PAUSE_UPGRADE.set(Arc::new(AtomicBool::new(false))).unwrap();

    main_window.on_pause_upgrade(|v: bool| {
        slint::spawn_local(async_compat::Compat::new(async move {
            if let Some(ac) = PAUSE_UPGRADE.get() {
                ac.store(v, Ordering::Relaxed);
            }
        }))
        .expect("TODO: panic message");
    });

    let app = main_window.clone_strong();
    let app2 = main_window.clone_strong();
    slint::spawn_local(async move {
        business_logic::handle(app).await;
    })
    .expect("TODO: panic message");

    slint::spawn_local(async_compat::Compat::new(async move {
        task::task(&mut rx, app2).await;
    }))
    .unwrap();

    main_window.show().unwrap();
    // slint::run_event_loop_until_quit().unwrap();
    tokio::task::block_in_place(slint::run_event_loop).unwrap();

    info!("程序结束");
    Ok(())
}
