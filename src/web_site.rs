use std::process::Command;

pub fn open_website(url:&str) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = Command::new("pwsh");
    let a = command.arg("-Command")
        .arg(format!("Start-Process {}", url))
        .status().expect("启动失败");

    if a.success() {
        println!("启动成功")
    }

    Ok(())
}