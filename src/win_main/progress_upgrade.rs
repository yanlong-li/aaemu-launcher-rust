use crate::win_main::{SafeHWND, PROGRESS_HWND};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::PROGRESS_CLASS;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW
    , HMENU, WINDOW_EX_STYLE
    , WS_ACTIVECAPTION, WS_CHILD, WS_VISIBLE,
};

pub fn create(hwndparent: HWND) -> HWND {
    PROGRESS_HWND
        .get_or_init(|| {
            SafeHWND(unsafe {
                CreateWindowExW(
                    WINDOW_EX_STYLE(0),
                    PCWSTR(PROGRESS_CLASS.0),
                    PCWSTR(std::ptr::null()),
                    WS_CHILD | WS_VISIBLE | WS_ACTIVECAPTION,
                    0,
                    600 - 10,
                    800,
                    10,
                    hwndparent,
                    HMENU::default(),
                    GetModuleHandleW(None).unwrap(),
                    Some(std::ptr::null_mut()),
                )
                .unwrap()
            })
        })
        .0
}
