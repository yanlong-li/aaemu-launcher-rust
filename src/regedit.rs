use std::env;
use std::fs::{File, remove_file};
use std::process::Command;

use byteorder::{LittleEndian, WriteBytesExt};
use encoding_rs::GBK;
use tracing::info;

pub fn get_reg_str() -> String {
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

pub fn detecting() -> bool {

    let output = Command::new("reg").arg("query").arg("HKEY_CLASSES_ROOT\\plaa\\shell\\open\\command").output().expect("查询失败");


    let mut ok = false;

    if output.status.success() {
        let binding = output.stdout.to_vec();
        let out_msg = GBK.decode(&*binding);
        info!("ok {:?}", out_msg.0);
        let program = env::current_exe().expect("获取路径失败");

        let s = program.to_str().unwrap();

        ok = out_msg.0.contains(&s);

        if !ok {
            info!("地址不匹配，重新注册")
        }
    } else {
        let binding = output.stderr.to_vec();
        let out_msg = GBK.decode(&*binding);
        info!("err {:?}", out_msg.0);
    }


    ok
}

pub fn register() -> bool {
    let root_path = env::current_exe().expect("获取当前路径失败").parent().expect("获取父级目录").to_str().expect("转换为字符串失败").to_string();

    let program = env::current_exe().expect("获取路径失败");

    let s = program.to_str().unwrap().replace("\\", "\\\\");

    let reg_str = get_reg_str().replace("{{program}}", &s);

    let reg_path = format!("{}\\protocol.reg", root_path);

    {
        let v: Vec<u16> = reg_str.encode_utf16().collect();
        let mut file = File::create(&reg_path).unwrap();
        file.write_u16::<LittleEndian>(0xFEFF).unwrap();
        for i in 0..v.len() {
            file.write_u16::<LittleEndian>(v[i]).unwrap();
        }
    }

    info!("{}", "导入注册表数据");
    let output = Command::new("reg")
        .arg("import")
        .arg(&reg_path)
        .output()
        .expect("导入注册表失败");


    // 将输出转换为字符串

    if output.status.success() {
        let binding = output.stdout.to_vec();
        let out_msg = GBK.decode(binding.as_ref());
        info!("{:?}", out_msg);
    } else {
        let binding = output.stderr.to_vec();
        let out_msg = GBK.decode(binding.as_ref());
        info!("{:?}", out_msg.0);
    }


    // 删除文件
    let res = remove_file(reg_path);
    info!("清理临时数据 {:?}", res);

    return output.status.success();
}