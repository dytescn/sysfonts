//! 真实路径冒烟测试：安装 → 列表 → 卸载，以及导出。

mod common;

use common::*;
use std::path::PathBuf;

fn pick_real_font() -> Option<(String, String)> {
    let v = do_list();
    let fonts = v["data"]["fonts"].as_array()?;
    for f in fonts {
        let path = f["path"].as_str()?;
        let ps = f["postscript_name"].as_str()?;
        let lower = path.to_lowercase();
        let is_truetype = lower.ends_with(".ttf")
            || lower.ends_with(".otf")
            || lower.ends_with(".ttc");
        if is_truetype && !ps.is_empty() && PathBuf::from(path).exists() {
            return Some((path.to_string(), ps.to_string()));
        }
    }
    None
}

fn copy_to_temp(src: &str, tmp: &std::path::Path) -> PathBuf {
    let src_path = PathBuf::from(src);
    let filename = src_path.file_name().expect("font path has no filename");
    let dst = tmp.join(filename);
    std::fs::copy(&src_path, &dst).expect("failed to copy font to temp");
    dst
}

#[test]
fn test_install_makes_font_available() {
    let (src, ps_name) = match pick_real_font() {
        Some(x) => x,
        None => {
            eprintln!("skip: no usable system font found");
            return;
        }
    };
    eprintln!("using font: {src} (ps={ps_name})");

    let tmp = temp_dir("roundtrip");
    let copy = copy_to_temp(&src, &tmp);
    let copy_str = copy.to_string_lossy().to_string();
    let filename = copy.file_name().unwrap();
    let installed_path = user_font_dir().join(filename);

    // 1. 安装
    let v = do_install(&copy_str, "user");
    assert_eq!(v["code"], 200, "install_font failed: {v}");
    eprintln!("install ok");

    // 2. 副本落地
    assert!(installed_path.exists(), "installed file missing");
    let src_size = std::fs::metadata(&copy).unwrap().len();
    let dst_size = std::fs::metadata(&installed_path).unwrap().len();
    assert_eq!(src_size, dst_size);

    // 3. 字体可用
    let after = do_list();
    let exists_after = after["data"]["fonts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["postscript_name"].as_str() == Some(ps_name.as_str()));
    assert!(exists_after);

    // 4. 卸载
    let v = do_uninstall(&ps_name, false);
    let code = v["code"].as_i64().unwrap();
    assert!(code == 200 || code == 500102);

    // 5. 清理
    if installed_path.exists() {
        let _ = std::fs::remove_file(&installed_path);
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_export_font_to_temp_dir() {
    let (src, ps_name) = match pick_real_font() {
        Some(x) => x,
        None => return,
    };
    eprintln!("using font: {src} (ps={ps_name})");

    let tmp = temp_dir("export");
    let dest_dir = tmp.join("out");
    let dest_dir_str = dest_dir.to_string_lossy().to_string();

    let v = do_export(&ps_name, &dest_dir_str);
    assert_eq!(v["code"], 200, "export_font failed: {v}");

    let exported = v["data"]["path"].as_str().expect("data.path");
    let exported_path = PathBuf::from(exported);
    assert!(exported_path.exists());

    let src_size = std::fs::metadata(&src).unwrap().len();
    let dst_size = std::fs::metadata(&exported_path).unwrap().len();
    assert_eq!(src_size, dst_size);

    eprintln!("export ok -> {exported}");
    let _ = std::fs::remove_dir_all(&tmp);
}
