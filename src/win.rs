use windows::{
    Win32::Foundation::*,
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{
        ShowWindow
        , SW_HIDE,
    },
};

use windows::{
    Win32::Foundation::*
    ,
    Win32::UI::WindowsAndMessaging::*,
};
use windows::Win32::System::Console::GetConsoleWindow;

pub fn win() {
    // 获取控制台窗口句柄
    let console_window: HWND = unsafe { GetConsoleWindow() };

    println!("{}",console_window.is_invalid());
    if !console_window.is_invalid() {
        // 隐藏控制台窗口
        unsafe {
            let res = ShowWindow(console_window, SW_HIDE);
            println!("是否关闭 {:?}",res);
        }
    }
}
