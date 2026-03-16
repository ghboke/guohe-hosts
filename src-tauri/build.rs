use std::env;

fn main() {
    // Rerun build script when icon changes so exe resource gets updated
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-env-changed=APP_ENABLE_UPDATE");

    let app_enable_update = env::var("APP_ENABLE_UPDATE").unwrap_or_else(|_| "true".to_string());
    println!("cargo:rustc-env=APP_ENABLE_UPDATE={app_enable_update}");

    let mut windows = tauri_build::WindowsAttributes::new();
    windows = windows.app_manifest(include_str!("guohe-hosts.exe.manifest"));
    windows = windows.window_icon_path("icons/icon.ico");

    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri build script");
}
