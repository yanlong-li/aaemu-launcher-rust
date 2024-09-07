use std::hash::Hash;

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW, SendMessageW, ShowWindow, SW_SHOW, WM_COMMAND, WM_DESTROY};

use crate::{db_check, protocol, regedit, site_link_url, system_config, trion_1_2, VERSION, web_site, WEBSITE_URL};
use crate::protocol::AuthToken;
use crate::win_main::WmCommand::Notice;

pub async fn handle(hwnd: HWND) {
    // region 业务逻辑

   site_link_url::handle().await;

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

    let res = protocol::handle().await;

    if res.is_err() {
        unsafe {
            MessageBoxW(hwnd, w!("本服非直接启动，请在官网点击“启动游戏”按钮。"), w!("游戏启动提示"), MB_OK);
            web_site::open_website(WEBSITE_URL);
            SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
        }
        return;
    }
    let auth_token = res.unwrap();

    if (!handle_version(auth_token.with_launcher_version).await) {
        unsafe {
            SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
        }
        return;
    }

    if !handle_db_check(hwnd, &auth_token).await {
        return;
    }
    handle_conf().await;

    // web_site::open_website(WEBSITE_URL);

    unsafe {
        // let text = crate::win_main::NOTICE_TEXT_HWND.get().unwrap().0;

        SendMessageW(hwnd, WM_COMMAND, WPARAM(Notice.into_usize()), LPARAM(1));
    }

    handle_launch(&auth_token).await;
    unsafe {
        SendMessageW(hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
    }
    // endregion
    return;
}

pub async fn handle_conf() {
    system_config::update().await;
}

pub async fn handle_db_check(hwnd: HWND, auth_token: &AuthToken) -> bool {
    if db_check::detect_db(auth_token.db_hash.as_ref()).is_err() {
        // web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
        unsafe {
            SendMessageW(hwnd, WM_COMMAND, WPARAM(super::win_main::WmCommand::UpgradeButton.into_usize()), LPARAM(1));
            SendMessageW(hwnd, WM_COMMAND, WPARAM(Notice.into_usize()), LPARAM(0));
        }
        return false;
    }
    return true;
}

pub async fn handle_version(with_launcher_version: u16) -> bool {
    if with_launcher_version > VERSION {
        unsafe {
            MessageBoxW(None, w!("当前版本太低，请更新到最新版！"), w!("版本错误"), MB_OK);
            web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
            SendMessageW(None, WM_DESTROY, WPARAM(0), LPARAM(0));
            return false;
        }
    }
    return true;
}

pub async fn handle_launch(auth_token: &AuthToken) {
    // let protocol_result = protocol::handle().await;
    trion_1_2::launch(auth_token).await;
    // match auth_token {
    //     Ok(auth_token) => {
    //         trion_1_2::launch(&auth_token);
    //     }
    //     Err(_) => {
    //         web_site::open_website(WEBSITE_URL).expect("无法启动浏览器");
    //         unsafe {
    //             SendMessageW(None, WM_DESTROY, WPARAM(0), LPARAM(0));
    //         }
    //     }
    // }
}