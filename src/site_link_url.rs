use std::env;
use windows::core::{Interface, HSTRING};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile, CLSCTX_ALL,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

pub async fn handle() -> Result<(), Box<dyn std::error::Error>> {
   handle_test().await
}

pub async fn handle_test() -> Result<(), Box<dyn std::error::Error>> {
    let homepath = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOMEPATH"))
        .expect("获取用户环境失败 HOMEPATH");

    let file_name = "plaa.lnk";

    // 创建一个包含目录和文件名的完整路径
    let file_path = format!("{}\\Desktop\\{}", homepath, file_name);

    unsafe {
        // 初始化 COM 库
        let hresult = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        if hresult.is_err() {
            return Err("CoInitializeEx failed".into());
        }
        // 创建 ShellLink 对象
        // let mut shell_link: Option<IShellLinkW> = None;

        let shell_link = CoCreateInstance(&ShellLink, None, CLSCTX_ALL);

        if shell_link.is_err() {
            println!("{:?}", shell_link.err());
            return Err("CoCreateInstance failed".into());
        }
        //
        let shell_link: IShellLinkW = shell_link.unwrap();

        // 使用 HSTRING 包装目标网址
        let target_url = HSTRING::from("https://plaa.top");

        // 设置目标网址
        shell_link.SetPath(&target_url)?;

        let target_dir = HSTRING::from("d:\\游戏\\aw\\ArcheWorld\\");
        // 设置起始目录
        shell_link.SetWorkingDirectory(&target_dir)?;


        let current_exe = env::current_exe()?.to_str().unwrap().to_string();

        let target_icon = HSTRING::from(current_exe);
        // 设置图标
        shell_link.SetIconLocation(&target_icon, 0)?;

        let target_name = HSTRING::from("AA");
        // 设置备注
        shell_link.SetDescription(&target_name)?;

        // 保存快捷方式
        let persist_file: IPersistFile = shell_link.cast()?;

        let target_save = HSTRING::from(&file_path);

        let result = persist_file.Save(&target_save, true);
        //
        if result.is_ok() {
            println!("Shortcut created successfully at {}", &file_path);
        } else {
            println!("发生错误 {}", result.unwrap_err());
        }

        CoUninitialize();
        Ok(())
    }
}
