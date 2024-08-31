// #![windows_subsystem = "windows"]

use std::thread::sleep;
use std::time::Duration;
use windows::core::w;
use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW};

mod trion_1_2;
mod web_site;

mod regedit;

mod win;
mod protocol;
mod cipher;

mod helper;

const WEBSITE_URL: &str = "https://aaemu.yanlongli.com";

const VERSION: u16 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !regedit::detecting() {
        if !regedit::register() {
            unsafe { MessageBoxW(None, w!("首次安装请在文件上右键“以管理员权限运行”"), w!("发生错误"), MB_OK); }
            return Ok(());
        }
    }

    let protocol_result = protocol::handle();

    match protocol_result {
        Ok(auth_token) => {
            if auth_token.with_launcher_version > VERSION {
                unsafe {
                    MessageBoxW(None, w!("当前版本太低，请更新到最新版！"), w!("版本错误"), MB_OK);
                }
            }

            trion_1_2::launch(&auth_token);
        }
        Err(_) => {
            web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
        }
    }


    // web_site::open_website(WEBSITE_URL);

    sleep(Duration::from_secs(5));
    Ok(())
}
