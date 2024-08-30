use std::env;
use std::fs::{read_to_string, remove_file, File};
use std::io::Read;
use std::process::Command;
use byteorder::{LittleEndian, WriteBytesExt};

pub fn get_reg_str() -> String {
    let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();


    let reg_result = read_to_string(format!("{}\\reg.reg", root_path));

    match reg_result {
        Ok(val) => {
            val
        }
        Err(_) => {
            String::from(r#"Windows Registry Editor Version 5.00

[HKEY_CLASSES_ROOT\plaa]
@="URL:Plaa Protocol"
"URL Protocol"=""

[HKEY_CLASSES_ROOT\plaa\shell]

[HKEY_CLASSES_ROOT\plaa\shell\open]

[HKEY_CLASSES_ROOT\plaa\shell\open\command]
@="\"{{program}}\" \"%1\""

"#)
        }
    }
}

pub fn register() {
    let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();

    let reg_str = get_reg_str();

    let program = env::current_exe().expect("获取路径失败");

    let reg_str = reg_str.replace("{{program}}", program.to_str().unwrap());

    let reg_path = format!("{}\\protocol.reg", root_path);

    {
        let v: Vec<u16> = reg_str.encode_utf16().collect();
        let mut file = File::create(&reg_path).unwrap();
        file.write_u16::<LittleEndian>(0xFEFF).unwrap();
        for i in 0..v.len() {
            file.write_u16::<LittleEndian>(v[i]).unwrap();
        }
    }

    println!("{}", "导入注册表数据");
    let output = Command::new("reg")
        .arg("import")
        .arg(&reg_path)
        .output()
        .expect("导入注册表失败");

    // 删除文件
    let res = remove_file(reg_path);

    println!("删除文件 {:?}",res)

}