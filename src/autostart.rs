//! Per-platform auto-start on login. `install()` / `uninstall()` are invoked
//! by the `--install` / `--uninstall` CLI flags.

use std::path::PathBuf;

const LABEL: &str = "com.samdev.claude-usage-bar";

/// Absolute path to the executable that auto-start should launch.
fn exe_path() -> PathBuf {
    std::env::current_exe().expect("current exe path is available")
}

#[cfg(target_os = "macos")]
fn current_uid() -> u32 {
    // `id -u` avoids a libc dependency just for getuid().
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

#[cfg(target_os = "macos")]
pub fn install() {
    use std::process::Command;

    let plist_dir = dirs::home_dir().unwrap().join("Library/LaunchAgents");
    let _ = std::fs::create_dir_all(&plist_dir);
    let plist_path = plist_dir.join(format!("{LABEL}.plist"));
    let exe = exe_path();
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>{LABEL}</string>
    <key>ProgramArguments</key><array><string>{}</string></array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><true/>
</dict>
</plist>
"#,
        exe.display()
    );
    std::fs::write(&plist_path, plist).expect("write LaunchAgent plist");
    let uid = format!("gui/{}", current_uid());
    let _ = Command::new("launchctl")
        .args(["bootstrap", &uid, plist_path.to_str().unwrap()])
        .status();
    println!("installed: {}", plist_path.display());
}

#[cfg(target_os = "macos")]
pub fn uninstall() {
    let plist_path = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist"));
    let uid = format!("gui/{}", current_uid());
    let _ = std::process::Command::new("launchctl")
        .args(["bootout", &uid, plist_path.to_str().unwrap()])
        .status();
    let _ = std::fs::remove_file(&plist_path);
    println!("uninstalled");
}

#[cfg(target_os = "linux")]
pub fn install() {
    let dir = dirs::home_dir().unwrap().join(".config/autostart");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("claude-usage-bar.desktop");
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Claude Usage Bar\nExec={}\nX-GNOME-Autostart-enabled=true\n",
        exe_path().display()
    );
    std::fs::write(&path, entry).expect("write autostart .desktop");
    println!("installed: {}", path.display());
}

#[cfg(target_os = "linux")]
pub fn uninstall() {
    let path = dirs::home_dir()
        .unwrap()
        .join(".config/autostart/claude-usage-bar.desktop");
    let _ = std::fs::remove_file(&path);
    println!("uninstalled");
}

#[cfg(target_os = "windows")]
pub fn install() {
    // HKCU Run key — value name = LABEL, value data = exe path.
    let status = std::process::Command::new("reg")
        .args([
            "add",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            LABEL,
            "/t",
            "REG_SZ",
            "/d",
            &exe_path().display().to_string(),
            "/f",
        ])
        .status();
    match status {
        Ok(s) if s.success() => println!("installed: HKCU Run\\{LABEL}"),
        _ => eprintln!("install failed — could not write the registry Run key"),
    }
}

#[cfg(target_os = "windows")]
pub fn uninstall() {
    let _ = std::process::Command::new("reg")
        .args([
            "delete",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            LABEL,
            "/f",
        ])
        .status();
    println!("uninstalled");
}
