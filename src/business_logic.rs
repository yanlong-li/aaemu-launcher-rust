use std::time::Duration;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, SendMessageW, ShowWindow, MB_OK, SW_SHOW, SW_SHOWNORMAL, WM_COMMAND, WM_DESTROY};
use crate::{db_check, protocol, regedit, system_config, trion_1_2, web_site, VERSION, WEBSITE_URL};

pub fn handle(hwnd: HWND) {
    // region 业务逻辑

    if !regedit::detecting() {
        if !regedit::register() {
            unsafe {
                MessageBoxW(hwnd, w!("首次安装请在文件上右键“以管理员权限运行”"), w!("发生错误"), MB_OK);
                SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
            }
            return;
        }
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
    };


    let _ = system_config::update();

    let protocol_result = protocol::handle();

    match protocol_result {
        Ok(auth_token) => {
            if auth_token.with_launcher_version > VERSION {
                unsafe {
                    MessageBoxW(None, w!("当前版本太低，请更新到最新版！"), w!("版本错误"), MB_OK);
                    web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
                    SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
                    return;
                }
            }

            unsafe {
                SendMessageW(hwnd, WM_COMMAND, WPARAM(super::win_main::WmCommand::UpgradeButton.into_usize()), LPARAM(1));
            }

            if db_check::handle(auth_token.db_hash.as_ref()).is_err() {
                web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
                unsafe {
                    SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
                }
                return;
            }

            trion_1_2::launch(&auth_token);
        }
        Err(_) => {
            web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
            unsafe {
                SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
            }
        }
    }


    // web_site::open_website(WEBSITE_URL);

    // endregion
    return;
}