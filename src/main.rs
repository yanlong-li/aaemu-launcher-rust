use std::env;
use std::env::args;
use std::thread::sleep;
use std::time::Duration;
use windows::core::{s, w};
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject, CREATE_NO_WINDOW};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxA, MessageBoxW, ShowWindow, MB_OK, SW_HIDE};

mod trion_1_2;
mod web_site;

mod regedit;

mod win;
mod protocol;
mod aes;

const WEBSITE_URL: &str = "https://aaemu.yanlongli.com";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    win::win();

    if !is_root() {
        println!("This program needs to be run as root.");
        println!("Try running with 'sudo' or as a root user.");
        std::process::exit(1);
    }

    regedit::register();

    let res = protocol::handle();

    println!("{:?}",res);

    // let (file, event) = trion_1_2::init_ticket("test3", "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")?;
    // trion_1_2::launch(file, event);
    // web_site::open_website(WEBSITE_URL);
    Ok(())
}

fn is_root() -> bool {
    env::var("USERNAME").unwrap_or_default() == "PC"
}
