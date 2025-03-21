use crate::win_main::WmCommand::PlayButton;
use crate::win_main::{SafeHWND, HBITMAP_BG, PLAY_GAME_BUTTON_HWND};
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
    BS_PUSHBUTTON, GWLP_WNDPROC, HMENU, IMAGE_BITMAP, LR_LOADFROMFILE, WINDOW_STYLE,
    WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE,
    WM_PAINT, WNDPROC, WS_CHILD, WS_VISIBLE,
};

pub fn create(hwndparent: HWND) {
    unsafe {
        PLAY_GAME_BUTTON_HWND.get_or_init(|| {
            let button = CreateWindowExW(
                Default::default(),
                w!("BUTTON"),
                w!("开始游戏"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                (800 - 300) / 2,
                420,
                300,
                74,
                hwndparent,
                HMENU(PlayButton as usize as *mut std::ffi::c_void),
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
pub static mut ACTIVE: bool = false;
pub extern "system" fn start_button_subclass_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => {
            println!("[开始按钮]鼠标激活");
        }
        WM_LBUTTONDOWN => {
            println!("[开始按钮][按钮按下]");
            unsafe {
                ACTIVE = true;
                let _ = InvalidateRect(hwnd, None, false);
            }
        }
        WM_LBUTTONUP => {
            println!("[开始按钮][按钮弹起]");
            unsafe {
                ACTIVE = false;
                let _ = InvalidateRect(hwnd, None, false);
            }
        }
        // WM_LBUTTONDBLCLK => {
        //     println!("[开始按钮][按钮点击]");
        // }
        // WM_CREATE => {
        //     println!("[开始按钮] 窗口创建成功");
        // },
        WM_PAINT => {
            println!("[开始按钮] 窗体绘制");
            brush(hwnd);
        }
        WM_ERASEBKGND => {
            println!("[开始按钮] 通知需要绘制窗口");
            brush(hwnd);
        }
        // WM_NCPAINT => {
        //     println!("[开始按钮] 通知绘制非客户区");
        // }
        // WM_SETFOCUS => {
        //     println!("[开始按钮] 获得焦点");
        // }
        // 160 => {
        //     println!("[开始按钮] 鼠标移动 {:?} {:?}", w_param, l_param);
        // } // 鼠标移动
        // 243 => {} // 点击后移动
        // 32 => {}
        // 132 => {}
        // 512 => {} // 移动
        _ => {
            // println!("[start_button_subclass_proc][{msg}]", msg = msg);
        }
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
    println!("开始按钮绘制 brush");
    unsafe {
        let bg_bmp = if ACTIVE {
            w!(r"./resources/play_button_active.bmp")
        } else {
            w!(r"./resources/play.bmp")
        };

        let backgroud_hbimap = HBITMAP(
            LoadImageW(
                GetModuleHandleW(None).unwrap(),
                PCWSTR(bg_bmp.as_ptr()),
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
        let old_bitmap = SelectObject(hdc_mem, backgroud_hbimap);
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
        .expect("开始按钮绘制失败");
        SelectObject(hdc_mem, old_bitmap);

        let _ = EndPaint(hwnd, &ps);
    }
}
