use crate::win_main::button_start::create;
use crate::win_main::WmCommand::{Exit, Notice, PlayButton, ShowPlayButton, StartUpgrade};
use std::sync::OnceLock;
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, RECT};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, BitBlt, CreateCompatibleDC, EndPaint, GetObjectW, SelectObject, BITMAP, HBITMAP,
    PAINTSTRUCT, SRCCOPY,
};
use windows::Win32::UI::Controls::PBM_SETRANGE;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, LoadImageW, MessageBoxW, SendMessageW, SetWindowTextW,
    ShowWindow, ES_CENTER, IMAGE_BITMAP, LR_LOADFROMFILE, MB_OK, SW_HIDE, SYSTEM_METRICS_INDEX,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_PAINT, WS_POPUP,
};
use windows::{
    core::PCWSTR,
    Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Controls::{InitCommonControls, PBM_SETPOS},
    Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, LoadCursorW, PostQuitMessage, RegisterClassW, CS_HREDRAW,
        CS_VREDRAW, HMENU, IDC_ARROW, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WNDCLASSW, WS_CHILD,
        WS_VISIBLE,
    },
};

static mut HBITMAP_BG: HBITMAP = HBITMAP(0 as *mut std::ffi::c_void);

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

        // 获取屏幕的宽度和高度
        let screen_width = GetSystemMetrics(SYSTEM_METRICS_INDEX(0)); // 0代表屏幕宽度
        let screen_height = GetSystemMetrics(SYSTEM_METRICS_INDEX(1)); // 1代表屏幕高度

        // 计算窗口的位置：居中显示
        let x_position = (screen_width - 800) / 2;
        let y_position = (screen_height - 600) / 2;

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            w!("随时删档跑路的上古"),
            WS_VISIBLE | WS_POPUP,
            // WS_OVERLAPPEDWINDOW,
            x_position,
            y_position,
            800,
            600,
            HWND::default(),
            HMENU::default(),
            h_instance,
            Some(std::ptr::null_mut()),
        )
        .unwrap();

        hwnd
    }
}

// 为 HWND 创建一个包装类型，并手动实现 Sync 和 Send
#[derive(Copy, Clone, Debug)]
pub struct SafeHWND(pub(crate) HWND);

// 手动实现 Send 和 Sync
unsafe impl Send for SafeHWND {}
unsafe impl Sync for SafeHWND {}

// 使用 OnceLock 来存储线程安全的 SafeHWND
static PROGRESS_HWND: OnceLock<SafeHWND> = OnceLock::new();
static UPGRADE_BUTTON_HWND: OnceLock<SafeHWND> = OnceLock::new();
pub(crate) static NOTICE_TEXT_HWND: OnceLock<SafeHWND> = OnceLock::new();
static PLAY_GAME_BUTTON_HWND: OnceLock<SafeHWND> = OnceLock::new();
static EXIT_GAME_BUTTON_HWND: OnceLock<SafeHWND> = OnceLock::new();

fn makelparam(low: u16, high: u16) -> LPARAM {
    LPARAM(((low as i32) | ((high as i32) << 16)) as isize)
}


#[repr(usize)]
pub enum WmCommand {
    Progress,
    PlayButton,
    UpgradeButton,
    StartUpgrade,
    Notice,
    Exit,
    ShowPlayButton,
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
                // 加载背景图片
                HBITMAP_BG = HBITMAP(
                    LoadImageW(
                        GetModuleHandleW(None).unwrap(),
                        PCWSTR(w!(r"./resources/background.bmp").as_ptr()),
                        IMAGE_BITMAP,
                        0,
                        0,
                        LR_LOADFROMFILE,
                    )
                    .unwrap()
                    .0,
                );

                button_exit::create(hwnd);

                SendMessageW(hwnd, WM_COMMAND, WPARAM(Notice.into_usize()), LPARAM(1));
            }
        }
        WM_LBUTTONDOWN => {
            println!("WM_LBUTTONDOWN 事件 {}", w_param.0);
        }
        // 自定义消息
        WM_COMMAND => {
            match w_param.0 {
                val if val == WmCommand::Progress.into_usize() => {
                    println!("触发 Progress");
                    unsafe {
                        SendMessageW(
                            PROGRESS_HWND.get().unwrap().0,
                            PBM_SETPOS,
                            WPARAM(l_param.0 as usize),
                            LPARAM(0),
                        );
                    }

                    if l_param.0 >= 100 {
                        unsafe {
                            MessageBoxW(None, w!("资源更新完成！"), w!("资源更新"), MB_OK);
                            let _ = ShowWindow(PROGRESS_HWND.get().unwrap().0, SW_HIDE);
                            SendMessageW(hwnd, WM_COMMAND, WPARAM(Notice.into_usize()), LPARAM(2));

                            SendMessageW(
                                hwnd,
                                WM_COMMAND,
                                WPARAM(ShowPlayButton.into_usize()),
                                LPARAM(1),
                            );
                        }
                    }
                }
                val if val == WmCommand::UpgradeButton.into_usize() => {
                    println!("触发 UpgradeButton");
                    button_upgrade::create(hwnd);
                }
                val if val == StartUpgrade.into_usize() => {
                    println!("触发 StartUpgrade");

                    let progress_hwnd = progress_upgrade::create(hwnd);

                    unsafe {
                        let _ = ShowWindow(UPGRADE_BUTTON_HWND.get().unwrap().0, SW_HIDE);
                        // 设置进度条范围及初始位置
                        // 设置进度条范围：最小值为 0，最大值为 100
                        SendMessageW(progress_hwnd, PBM_SETRANGE, WPARAM(0), makelparam(0, 100));
                        // 设置进度条的当前位置为 50
                        SendMessageW(progress_hwnd, PBM_SETPOS, WPARAM(0), LPARAM(0));
                    }

                    let res = super::SENDER
                        .get()
                        .clone()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .try_send((StartUpgrade.into_usize(), 0));
                    println!("{:?}", res);
                }
                val if val == Notice.into_usize() => {
                    let lpstr = match l_param {
                        LPARAM(0) => {
                            w!("当前有新版本需要更新，请点击开始更新按钮继续。")
                        }
                        LPARAM(1) => {
                            w!("游戏启动中，请稍后")
                        }
                        LPARAM(2) => {
                            w!("游戏资源已更新完成，您可以开始游戏啦~")
                        }
                        _ => {
                            w!("Notice")
                        }
                    };

                    println!("触发 Notice");
                    let notice_hwnd = NOTICE_TEXT_HWND.get_or_init(|| {
                        SafeHWND(unsafe {
                            CreateWindowExW(
                                Default::default(),
                                w!("STATIC"),
                                lpstr,
                                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(ES_CENTER as u32),
                                220,
                                420,
                                360,
                                30,
                                hwnd,
                                None,
                                GetModuleHandleW(None).unwrap(),
                                None,
                            )
                            .unwrap()
                        })
                    });

                    unsafe {
                        SetWindowTextW(notice_hwnd.0, lpstr).expect("TODO: panic message");
                    }
                }
                val if val == PlayButton.into_usize() => {
                    println!("触发 PlayButton");

                    let res = super::SENDER
                        .get()
                        .unwrap()
                        .lock()
                        .unwrap()
                        .try_send((PlayButton.into_usize(), 0));

                    println!("{:?}", res);
                }
                val if val == Exit.into_usize() => unsafe {
                    PostQuitMessage(0);
                },
                val if val == ShowPlayButton.into_usize() => unsafe {
                    create(hwnd);

                    NOTICE_TEXT_HWND.get().and_then(|no| {
                        let _ = ShowWindow(no.0, SW_HIDE);
                        Some(no)
                    });
                },
                _ => {
                    println!("没有处理的事件 {}", w_param.0);
                }
            }
        }
        // 窗口销毁
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
        },
        // 重新绘制窗口
        WM_PAINT => {
            println!("主窗体绘制");
            unsafe {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                // 获取窗口客户区尺寸
                let mut client_rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut client_rect);

                let hdc_mem = CreateCompatibleDC(hdc);
                let old_bitmap = SelectObject(hdc_mem, HBITMAP_BG);
                let mut bitmap = BITMAP::default();
                GetObjectW(
                    HBITMAP_BG,
                    std::mem::size_of::<BITMAP>() as i32,
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
                .expect("主窗口绘制失败");
                SelectObject(hdc_mem, old_bitmap);
                let _ = EndPaint(hwnd, &ps);
            }
        }
        _ => return unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }

    LRESULT(0)
}

mod button_exit;
mod button_start;
mod button_upgrade;
mod progress_upgrade;
