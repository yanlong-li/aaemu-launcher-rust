#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use std::{env};
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

mod window;

mod uac;

const WEBSITE_URL: &str = "https://plaa.top";

const VERSION: u16 = 3;

static SENDER: OnceLock<Arc<Mutex<Sender<Task>>>> = OnceLock::new();
static PAUSE_UPGRADE: OnceLock<Arc<AtomicBool>> = OnceLock::new();

#[derive(Debug)]
enum Task {
    Progress(f64),
    Message(String, String, MessageActions),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    info!("程序启动...");

    let (tx, mut rx) = channel::<Task>(1);
    SENDER.set(Arc::new(Mutex::new(tx.clone()))).unwrap();

    PAUSE_UPGRADE.set(Arc::new(AtomicBool::new(false))).unwrap();

    let app = window::create()?;

    slint::spawn_local(async move {
        business_logic::handle(&app).await;
        task::handle(&mut rx, &app).await;
    })
    .unwrap();

    // slint::run_event_loop_until_quit().unwrap();
    tokio::task::block_in_place(slint::run_event_loop).unwrap();

    info!("程序结束");
    Ok(())
}
