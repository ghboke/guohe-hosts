use std::path::PathBuf;

fn windows_dir() -> String {
    std::env::var("SystemRoot")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| std::env::var("WINDIR").ok().filter(|v| !v.trim().is_empty()))
        .unwrap_or_else(|| r"C:\Windows".to_string())
}

pub fn system_hosts_path() -> String {
    PathBuf::from(windows_dir())
        .join("System32")
        .join("drivers")
        .join("etc")
        .join("hosts")
        .to_string_lossy()
        .to_string()
}
