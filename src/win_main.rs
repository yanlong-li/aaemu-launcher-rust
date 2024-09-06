use std::sync::OnceLock;

use windows::{
    core::PCWSTR,
    Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Controls::{InitCommonControls, PBM_SETPOS, PROGRESS_CLASS},
    Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
        DefWindowProcW, HMENU, IDC_ARROW, LoadCursorW,
        PostQuitMessage, RegisterClassW, WM_CREATE, WM_DESTROY, WNDCLASSW, WS_CHILD
        , WS_VISIBLE,
    }
    ,
};
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HINSTANCE};
use windows::Win32::Graphics::Gdi::{BeginPaint, CreateSolidBrush, EndPaint, FillRect, PAINTSTRUCT};
use windows::Win32::UI::Controls::PBM_SETRANGE;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WINDOW_EX_STYLE, WM_COMMAND, WM_PAINT, WS_ACTIVECAPTION, WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_SYSMENU};

use crate::win_main::WmCommand::{Notice, StartUpgrade};

pub async fn handle() -> HWND {
    unsafe {
        // 初始化通用控件
        InitCommonControls();
        // 创建主窗口
        let h_instance = GetModuleHandleW(None).unwrap();
        let class_name = w!("随时删档跑路的上古");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            hInstance: HINSTANCE::from(h_instance),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            w!("随时删档跑路的上古"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            // WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            600,
            HWND::default(),
            HMENU::default(),
            h_instance,
            Some(std::ptr::null_mut()),
        ).unwrap();

        hwnd
    }
}

// 为 HWND 创建一个包装类型，并手动实现 Sync 和 Send
#[derive(Copy, Clone, Debug)]
struct SafeHWND(HWND);

// 手动实现 Send 和 Sync
unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}

// 使用 OnceLock 来存储线程安全的 SafeHWND
static PROGRESS_HWND: OnceLock<SafeHWND> = OnceLock::new();

// 用于存储进度条控件的句柄
// static PROGRESS_HWND: OnceLock<HWND> = OnceLock::new();
// 定义 MAKELPARAM 的函数
fn makelparam(low: u16, high: u16) -> LPARAM {
    LPARAM(((low as i32) | ((high as i32) << 16)) as isize)
}

enum IDC {
    IdcButton = 1001,
    IdcStaticText,
}


#[repr(usize)]
pub enum WmCommand {
    Progress,
    PlayButton,
    UpgradeButton,
    StartUpgrade,
    Notice,
}

impl WmCommand {
    pub(crate) fn into_usize(self) -> usize {
        self as usize
    }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        // 窗口创建
        WM_CREATE => {
            unsafe {
                // endregion
                // region text
                // Create static text
                CreateWindowExW(
                    Default::default(),
                    w!("STATIC"),
                    w!("PLAA"),
                    WS_CHILD | WS_VISIBLE | super::win32api::SS_CENTER,
                    10,
                    10,
                    360,
                    30,
                    hwnd,
                    None,
                    GetModuleHandleW(None).unwrap(),
                    None,
                );
                // endregion

                // super::business_logic::handle(hwnd)
            }
        }
        // 自定义消息
        WM_COMMAND => {
            match w_param.0 {
                val if val == WmCommand::Progress.into_usize() => {
                    println!("触发 Progress");
                    unsafe {
                        SendMessageW(PROGRESS_HWND.get().unwrap().0, PBM_SETPOS, WPARAM(l_param.0 as usize), LPARAM(0));
                    }
                }
                val if val == WmCommand::UpgradeButton.into_usize() => {
                    println!("触发 UpgradeButton");
                    unsafe {
                        CreateWindowExW(
                            Default::default(),
                            w!("BUTTON"),
                            w!("开始更新"),
                            WS_CHILD | WS_VISIBLE,
                            10,
                            400,
                            360,
                            30,
                            hwnd,
                            HMENU(WmCommand::StartUpgrade as usize as *mut std::ffi::c_void),
                            GetModuleHandleW(None).unwrap(),
                            None,
                        );
                    }
                }
                val if val == WmCommand::StartUpgrade.into_usize() => {
                    println!("触发 StartUpgrade");

                    let progress_hwnd = PROGRESS_HWND.get_or_init(|| {
                        SafeHWND(unsafe {
                            CreateWindowExW(
                                WINDOW_EX_STYLE(0),
                                PCWSTR(PROGRESS_CLASS.0),
                                PCWSTR(std::ptr::null()),
                                WS_CHILD | WS_VISIBLE | WS_ACTIVECAPTION,
                                10,
                                520,
                                360,
                                30,
                                hwnd,
                                HMENU::default(),
                                GetModuleHandleW(None).unwrap(),
                                Some(std::ptr::null_mut()),
                            ).unwrap()
                        })
                    }).0;

                    unsafe {
                        // 设置进度条范围及初始位置
                        // 设置进度条范围：最小值为 0，最大值为 100
                        SendMessageW(progress_hwnd, PBM_SETRANGE, WPARAM(0), makelparam(0, 100));
                        // 设置进度条的当前位置为 50
                        SendMessageW(progress_hwnd, PBM_SETPOS, WPARAM(0), LPARAM(0));
                    }

                    let rc = super::SENDER.get();

                    let rl = rc.clone();

                    let rm = rl.unwrap().lock();

                    let s = rm.unwrap().try_send((StartUpgrade.into_usize(), 0));

                    println!("{:?}",s);


                    // tokio::spawn(async {
                    //     let rc = super::SENDER.get();
                    //
                    //     match rc {
                    //         None => {}
                    //         Some(mut ra) => {
                    //             let rl = ra.clone();
                    //
                    //             let rm = rl.lock();
                    //
                    //             match rm {
                    //                 Ok(rs) => {
                    //                     for i in 0..101 {
                    //                         sleep(std::time::Duration::from_secs(1));
                    //                         println!("发送进度 {}", i);
                    //                         tokio::spawn(rs.send((WmCommand::Progress as usize, i)));
                    //                     }
                    //                 }
                    //                 Err(_) => {}
                    //             }
                    //         }
                    //     };
                    // });
                }
                val if val == Notice.into_usize() => {
                    println!("触发 Notice");

                    unsafe {
                        CreateWindowExW(
                            Default::default(),
                            w!("STATIC"),
                            w!("当前有新版本需要更新，请点击开始更新按钮继续。"),
                            WS_CHILD | WS_VISIBLE | super::win32api::SS_CENTER,
                            10,
                            50,
                            360,
                            60,
                            hwnd,
                            None,
                            GetModuleHandleW(None).unwrap(),
                            None,
                        );
                    }
                }
                _ => {
                    println!("没有处理的事件 {}", w_param.0);
                }
            }
        }
        // 窗口销毁
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
        }
        // 重新绘制窗口
        WM_PAINT => {
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);
                let brush = CreateSolidBrush(COLORREF(0xFFFFFF)); // 白色
                FillRect(hdc, &ps.rcPaint, brush);
                EndPaint(hwnd, &ps);
            }
        }
        _ => return unsafe {
            DefWindowProcW(hwnd, msg, w_param, l_param)
        },
    }

    LRESULT(0)
}

fn loword(value: usize) -> u16 {
    (value & 0xFFFF) as u16
}

fn hiword(value: usize) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}