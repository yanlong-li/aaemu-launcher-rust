use std::{env, ptr};
use std::mem::size_of;
use std::os::windows::process::CommandExt;

use rand::Rng;
use rc4::{KeyInit, Rc4, StreamCipher};
use windows::core::w;
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Memory::{CreateFileMappingW, FILE_MAP_ALL_ACCESS, MapViewOfFile, PAGE_READWRITE};
use windows::Win32::System::Threading::CreateEventW;
use windows::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW};

use crate::protocol::AuthToken;

const ARCHEAGE: &str = "\\archeage.exe";
const SUB_DIR: &str = "\\bin32";

pub fn init_ticket(username: &str, password: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut encryption_key = [0u8; 8];

    // 创建随机数生成器
    let mut rng = rand::thread_rng();

    // 填充字节数组
    rng.fill(&mut encryption_key);

    // 打印生成的随机字节数组（这里将其转为十六进制字符串以便查看）
    println!("随机Key: {:?}", hex::encode(encryption_key));


    // Step 1: Set up SECURITY_ATTRIBUTES for handle inheritance
    let mut sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: true.into(),
    };

    // Step 2: Define the maximum size for the file mapping
    let mut ticket_data = format!("TFIRdGVzdA==\n<?xml version=\"1.0\" encoding=\"UTF - 8\" standalone=\"yes\"?><authTicket version=\"1.2\"><storeToken>1</storeToken><client>PLAA</client><username>{}</username><password>{}</password></authTicket>", username, password).into_bytes(); // your encrypted data


    let mut rc4 = Rc4::new(&encryption_key.into());

    rc4.apply_keystream(&mut ticket_data);

    let max_map_size = ticket_data.len() as u32 + 12; // 8key + 4 len

    // Step 3: Create the file mapping object
    // let map_name = to_wide_string("archeage_auth_ticket_map");
    let file_map_handle = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE, // No associated file
            Some(&mut sa),
            PAGE_READWRITE,
            0,
            max_map_size,
            w!("archeage_auth_ticket_map"),
        )
    }?;

    if file_map_handle.is_invalid() {
        eprintln!("Failed to create file mapping object.");
        return Ok((0, 0));
    }

    // Step 4: Create a view of the file mapping
    let file_map_view = unsafe {
        MapViewOfFile(
            file_map_handle,
            FILE_MAP_ALL_ACCESS,
            0,
            0,
            max_map_size as usize,
        )
    };

    if file_map_view.Value.is_null() {
        eprintln!("Failed to map view of file.");
        return Ok((0, 0));
    }

    // Step 5: Copy data to the mapped memory
    unsafe {
        let dest = std::slice::from_raw_parts_mut(file_map_view.Value as *mut u8, max_map_size as usize);
        dest[..8].copy_from_slice(&encryption_key);

        let ticket_len: u32 = ticket_data.len() as u32;
        let tkl_byte = ticket_len.to_le_bytes();
        dest[8..12].copy_from_slice(&tkl_byte);
        dest[12..max_map_size as usize].copy_from_slice(&ticket_data);
    }

    // Step 6: Create an event object
    // let event_name = to_wide_string("archeage_auth_ticket_event");
    let event_handle = unsafe {
        CreateEventW(Some(&mut sa), true, false, w!("archeage_auth_ticket_event"))
    }?;

    if event_handle == HANDLE::default() {
        eprintln!("Failed to create event object.");
        return Ok((0, 0));
    }

    println!("文件映射创建成功");
    Ok((file_map_handle.0 as usize, event_handle.0 as usize))
}

pub(crate) fn launch(auth_token: &AuthToken) {
    let (p0, p1) = init_ticket(&auth_token.username, &auth_token.password).expect("初始化令牌失败");

    let handle_args = format!("-t +auth_ip {:} -auth_port {:} -handle {:08X}:{:08X} -lang zh_cn +acpxmk", auth_token.server, auth_token.port, p0, p1);

    println!("{:?}", handle_args);

    let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();

    let mut exe_path = format!("{}{}{}", root_path, SUB_DIR, ARCHEAGE);


    if !std::path::Path::exists(exe_path.as_ref()) {
        unsafe { MessageBoxW(None, w!("找不到游戏程序，请将启动器放置在游戏目录"), w!("发生错误"), MB_OK); }
        return;
    }


    let mut command = std::process::Command::new(exe_path);
    command.raw_arg(handle_args);

    // 启动程序并等待它完成
    let status = command.status().expect("Failed to start process");

    // 检查程序的退出状态
    if status.success() {
        println!("程序启动成功");
    } else {
        eprintln!("程序启动失败: {:?}", status);
    }
}