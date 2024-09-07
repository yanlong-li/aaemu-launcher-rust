use std::fs::File;
use std::io::Write;
use std::path::Path;

pub async fn handle() -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"[{000214A0-0000-0000-C000-000000000046}]
Prop3=19,11
[InternetShortcut]
IDList=
URL=https://plaa.top/
"#;

    let file_name = "启动游戏.url";

    // 检查文件是否存在
    if Path::new(file_name).exists() {
        println!("文件已存在，覆盖内容...");
    } else {
        println!("文件不存在，创建并写入...");
    }

    // 打开文件进行写入（创建或覆盖）
    let mut file = File::create(file_name)?;
    file.write_all(content.as_bytes())?;

    println!("操作完成！");
    Ok(())
}