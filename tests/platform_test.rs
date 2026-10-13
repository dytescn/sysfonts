//! 平台实现测试：安装 → 列出 → 卸载，用系统已有字体做。
//!
//! 注意：这些测试会真正修改系统字体目录（user scope）。
//! 在没有权限的 CI 上会失败，可用 `cargo test -- --ignored` 手动运行。

mod common;

use common::*;
use std::path::PathBuf;

/// 从系统字体列表里找一个真实存在的字体文件路径。
fn pick_a_real_font() -> Option<(String, String)> {
    let v = do_list();
    let fonts = v["data"]["fonts"].as_array()?;
    for f in fonts {
        let path = f["path"].as_str()?;
        let ps = f["postscript_name"].as_str();
        if PathBuf::from(path).exists() {
            return Some((path.to_string(), ps.unwrap_or("").to_string()));
        }
    }
    None
}

// ============================================================
// Linux / Unix
// ============================================================

#[cfg(not(any(windows, target_os = "macos")))]
#[test]
#[ignore = "会写入 ~/.local/share/fonts 并刷新 fontconfig，默认不跑"]
fn test_unix_install_list_uninstall_roundtrip() {
    let (src_path, _) = match pick_a_real_font() {
        Some(x) => x,
        None => {
            eprintln!("no real font found, skipping");
            return;
        }
    };

    // 用一个临时目录作为源，避免覆盖原字体
    let tmp = temp_dir("unix_roundtrip");
    let src = PathBuf::from(&src_path);
    let dst = tmp.join(src.file_name().unwrap());
    std::fs::copy(&src, &dst).unwrap();

    // 1. 安装
    let v = do_install(dst.to_str().unwrap(), "user");
    assert_eq!(v["code"], 200, "install failed: {v}");

    // 2. 列表里应该能查到
    let v = do_list();
    let found = v["data"]["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["path"].as_str() == Some(dst.to_str().unwrap()));
    assert!(found, "installed font not in list");

    // 3. 卸载（只取消注册，保留文件）
    let ps = v["data"]["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"].as_str() == Some(dst.to_str().unwrap()))
        .and_then(|f| f["postscript_name"].as_str())
        .map(|s| s.to_string());
    if let Some(ps) = ps {
        let v = do_uninstall(&ps, false);
        assert_eq!(v["code"], 200, "uninstall failed: {v}");
    } else {
        eprintln!("no postscript name, skipping uninstall check");
    }

    let _ = std::fs::remove_dir_all(&tmp);
}

// ============================================================
// macOS
// ============================================================

#[cfg(target_os = "macos")]
#[test]
#[ignore = "会写入 ~/Library/Fonts，默认不跑"]
fn test_macos_install_list_uninstall_roundtrip() {
    let (src_path, _) = match pick_a_real_font() {
        Some(x) => x,
        None => return,
    };

    let tmp = temp_dir("macos_roundtrip");
    let src = PathBuf::from(&src_path);
    let dst = tmp.join(src.file_name().unwrap());
    std::fs::copy(&src, &dst).unwrap();

    let v = do_install(dst.to_str().unwrap(), "user");
    assert_eq!(v["code"], 200, "install failed: {v}");

    let v = do_list();
    let found = v["data"]["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["path"].as_str() == Some(dst.to_str().unwrap()));
    assert!(found, "installed font not in list");

    let _ = std::fs::remove_dir_all(&tmp);
}

// ============================================================
// Windows
// ============================================================

#[cfg(windows)]
#[test]
#[ignore = "会写入 %LOCALAPPDATA%\\Microsoft\\Windows\\Fonts 并改注册表，默认不跑"]
fn test_windows_install_list_uninstall_roundtrip() {
    let (src_path, _) = match pick_a_real_font() {
        Some(x) => x,
        None => return,
    };

    let tmp = temp_dir("windows_roundtrip");
    let src = PathBuf::from(&src_path);
    let dst = tmp.join(src.file_name().unwrap());
    std::fs::copy(&src, &dst).unwrap();

    let v = do_install(dst.to_str().unwrap(), "user");
    assert_eq!(v["code"], 200, "install failed: {v}");

    let v = do_list();
    let found = v["data"]["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["path"].as_str() == Some(dst.to_str().unwrap()));
    assert!(found, "installed font not in list");

    let _ = std::fs::remove_dir_all(&tmp);
}
