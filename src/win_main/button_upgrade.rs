use crate::win_main::WmCommand::StartUpgrade;
use crate::win_main::{SafeHWND, HBITMAP_BG, UPGRADE_BUTTON_HWND};
use std::sync::OnceLock;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, EndPaint, GetObjectW, InvalidateRect, SelectObject,
    BITMAP, HBITMAP, PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, CreateWindowExW, GetWindowLongPtrW, LoadImageW, SetWindowLongPtrW,
    GWLP_WNDPROC, HMENU, IMAGE_BITMAP, LR_LOADFROMFILE, WM_ERASEBKGND, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_PAINT, WNDPROC, WS_CHILD, WS_VISIBLE,
};

pub fn create(hwndparent: HWND) {
    unsafe {
        UPGRADE_BUTTON_HWND.get_or_init(|| {
            let button = CreateWindowExW(
                Default::default(),
                w!("BUTTON"),
                w!("开始更新"),
                WS_CHILD | WS_VISIBLE,
                (800 - 300) / 2,
                420,
                300,
                74,
                hwndparent,
                HMENU(StartUpgrade as usize as *mut std::ffi::c_void),
                GetModuleHandleW(None).unwrap(),
                None,
            )
            .unwrap();

            ORIGINAL_BUTTON_PROC
                .set(std::mem::transmute(GetWindowLongPtrW(button, GWLP_WNDPROC)))
                .expect("TODO: panic message");

            SetWindowLongPtrW(button, GWLP_WNDPROC, start_button_subclass_proc as isize);

            SafeHWND(button)
        });
    }
}

pub static ORIGINAL_BUTTON_PROC: OnceLock<WNDPROC> = OnceLock::new();
pub extern "system" fn start_button_subclass_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => unsafe {
            let _ = InvalidateRect(hwnd, None, false);
        },
        WM_LBUTTONUP => {
            unsafe {
                let _ = InvalidateRect(hwnd, None, false);
            }
        }
        WM_PAINT => {
            brush(hwnd);
        }
        WM_ERASEBKGND => {
            brush(hwnd);
        }
        _ => {}
    }
    ORIGINAL_BUTTON_PROC
        .get()
        .and_then(|original_proc| unsafe {
            Some(CallWindowProcW(*original_proc, hwnd, msg, w_param, l_param))
        })
        .unwrap();
    LRESULT(1)
}

pub fn brush(hwnd: HWND) {
    // return;
    unsafe {
        let bg = HBITMAP(
            LoadImageW(
                GetModuleHandleW(None).unwrap(),
                PCWSTR(w!(r"./resources/upgrade.bmp").as_ptr()),
                IMAGE_BITMAP,
                0,
                0,
                LR_LOADFROMFILE,
            )
            .unwrap()
            .0,
        );

        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);
        let hdc_mem = CreateCompatibleDC(hdc);
        let old_bitmap = SelectObject(hdc_mem, bg);
        let mut bitmap = BITMAP::default();
        GetObjectW(
            HBITMAP_BG,
            size_of::<BITMAP>() as i32,
            Option::from(&mut bitmap as *mut BITMAP as *mut std::ffi::c_void),
        );

        BitBlt(
            hdc,
            0,
            0,
            bitmap.bmWidth,
            bitmap.bmHeight,
            hdc_mem,
            0,
            0,
            SRCCOPY,
        )
        .expect("绘制失败");
        SelectObject(hdc_mem, old_bitmap);

        let _ = EndPaint(hwnd, &ps);
    }
}
