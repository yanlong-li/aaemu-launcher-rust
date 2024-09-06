use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG, WINDOW_STYLE};

pub const SS_CENTER: WINDOW_STYLE = WINDOW_STYLE(1);


pub fn handle_msg() {
    // 消息循环
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, HWND::default(), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}