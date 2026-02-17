fn main() {
    // Rerun build script when icon changes so exe resource gets updated
    println!("cargo:rerun-if-changed=icons/icon.ico");

    let mut windows = tauri_build::WindowsAttributes::new();
    windows = windows.app_manifest(include_str!("guohe-hosts.exe.manifest"));
    windows = windows.window_icon_path("icons/icon.ico");

    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri build script");
}
