#![windows_subsystem = "windows"]

use std::thread::sleep;
use std::time::Duration;
use windows::core::w;
use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW};
mod trion_1_2;
mod web_site;

mod regedit;

mod win;
mod protocol;
mod aes;

const WEBSITE_URL: &str = "https://aaemu.yanlongli.com";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !regedit::detecting() {
        if !regedit::register() {
            unsafe { MessageBoxW(None, w!("首次安装请在文件上右键“以管理员权限运行”"), w!("发生错误"), MB_OK); }
            return Ok(());
        }
    }

    let res = protocol::handle();

    if res.is_err() {
        web_site::open_website(WEBSITE_URL);
    } else {
        let (file, event) = trion_1_2::init_ticket("test3", "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")?;
        trion_1_2::launch(file, event);
    }

    println!("{:?}", res);


    // web_site::open_website(WEBSITE_URL);

    sleep(Duration::from_secs(5));
    Ok(())
}
