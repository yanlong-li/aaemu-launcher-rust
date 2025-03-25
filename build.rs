use std::env;

fn main() {
    // 获取构建模式
    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));

    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let library_paths = std::collections::HashMap::from([(
        "sleek".to_string(),
        manifest_dir.join("sleek-ui/sleek.slint"),
    )]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library_paths);

    slint_build::compile_with_config("ui/app-window.slint", config).unwrap();

    if profile == "release" {
        println!("cargo:rerun-if-changed=app.manifest");
        println!("cargo:rerun-if-changed=app.ico");
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.set_icon("app.ico");
        // **应用名称、公司信息、版本信息**
        res.set("FileDescription", "上古世纪 PLAA 专用启动器"); // 应用描述
        res.set("ProductName", "PLAA 启动器"); // 应用名称
        res.set("CompanyName", "plaa.top"); // 作者/公司信息
                                            // res.set("FileVersion", "1.0.0.0"); // 文件版本
                                            // res.set("ProductVersion", "1.0.0.0"); // 产品版本
        res.set("LegalCopyright", "Copyright © 2025 plaa.top"); // 版权信息
        res.set_language(0x0804);
        res.compile().unwrap();

        // 调用 signtool 对 EXE 文件进行签名
        let status = std::process::Command::new("signtool")
            .args(&[
                "sign",
                "/f",
                "plaa-iis-0325164928.pfx",
                "/p",
                "123456",
                "/fd",
                "SHA256",
                "/t",
                "http://timestamp.digicert.com",
                "/v",
                "target/release/Launcher.exe",
            ])
            .status()
            .expect("签名操作失败");

        if !status.success() {
            panic!("签名失败");
        }
    }
}
