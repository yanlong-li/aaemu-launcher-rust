
fn main() {

    let manifest_dir = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let library_paths = std::collections::HashMap::from([(
        "sleek".to_string(),
        manifest_dir.join("sleek-ui/sleek.slint"),
    )]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library_paths);

    slint_build::compile_with_config("ui/app-window.slint",config).unwrap();

    // slint_build::compile("ui/popup.slint").unwrap();

    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rustc-link-arg=app.res");
}