fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    // 使用 windres 工具将 Manifest 嵌入到可执行文件中
    // std::process::Command::new("windres")
    //     .args(&["app.rc", "-o", "app.res"])
    //     .status()
    //     .expect("failed to execute windres");

    println!("cargo:rustc-link-arg=app.res");
    // 使用 `cc` crate 编译 .res 文件
    // cc::Build::new()
    //     .object("app.res") // 使用 object() 方法指向正确的 .res 文件
    //     .compile("app-res");
}