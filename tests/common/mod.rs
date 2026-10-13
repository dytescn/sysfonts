//! 测试辅助：封装 FFI 调用、JSON 解析、临时文件管理。

#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use serde_json::Value;

use sysfonts::fonts::fonts::{
    post_export_font, post_install_font, post_list_system_fonts, post_uninstall_font,
};
use sysfonts::utils::cjson::JSON_OUT_BUF_SIZE;

/// 分配一个和桥接层一致的输出缓冲区，调用 f，把结果转成 String。
pub fn call_ffi(
    json_in: Option<&str>,
    f: unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int,
) -> String {
    let c_in = json_in.map(|s| CString::new(s).expect("json_in contains null byte"));
    let in_ptr = match &c_in {
        Some(c) => c.as_ptr(),
        None => std::ptr::null(),
    };

    let mut buf = vec![0u8; JSON_OUT_BUF_SIZE];
    let out_ptr = buf.as_mut_ptr() as *mut c_char;

    let rc = unsafe { f(in_ptr, out_ptr) };
    assert_eq!(rc, 0, "FFI returned non-zero rc={rc}");

    let s = unsafe { CStr::from_ptr(out_ptr) }
        .to_string_lossy()
        .into_owned();
    s
}

/// 无参调用（list_system_fonts 会忽略 json_in）。
pub fn call_ffi_no_arg(
    f: unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int,
) -> String {
    call_ffi(None, f)
}

pub fn parse_and_assert_code(raw: &str, expected_code: i64) -> Value {
    let v: Value = serde_json::from_str(raw)
        .unwrap_or_else(|e| panic!("FFI returned invalid JSON: {e}\nraw={raw}"));
    let code = v
        .get("code")
        .and_then(|c| c.as_i64())
        .unwrap_or_else(|| panic!("missing 'code' field in: {raw}"));
    assert_eq!(code, expected_code, "unexpected code, raw={raw}");
    v
}

pub fn do_install(font_path: &str, scope: &str) -> Value {
    let req = serde_json::json!({
        "font_path": font_path,
        "scope": scope,
    })
    .to_string();
    let raw = call_ffi(Some(&req), post_install_font);
    serde_json::from_str(&raw).expect("invalid json")
}

pub fn do_list() -> Value {
    do_list_with_limit(100)
}

pub fn do_list_with_limit(limit: usize) -> Value {
    let req = serde_json::json!({ "limit": limit }).to_string();
    let raw = call_ffi(Some(&req), post_list_system_fonts);
    serde_json::from_str(&raw).expect("invalid json")
}

pub fn do_uninstall(postscript_name: &str, delete_file: bool) -> Value {
    let req = serde_json::json!({
        "postscript_name": postscript_name,
        "delete_file": delete_file,
    })
    .to_string();
    let raw = call_ffi(Some(&req), post_uninstall_font);
    serde_json::from_str(&raw).expect("invalid json")
}

pub fn do_export(postscript_name: &str, dest_dir: &str) -> Value {
    let req = serde_json::json!({
        "postscript_name": postscript_name,
        "dest_dir": dest_dir,
    })
    .to_string();
    let raw = call_ffi(Some(&req), post_export_font);
    serde_json::from_str(&raw).expect("invalid json")
}

pub fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("sysfonts_test_{tag}_{pid}_{nanos}"));
    std::fs::create_dir_all(&p).expect("failed to create temp dir");
    p
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn user_font_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".local/share/fonts")
}

#[cfg(windows)]
pub fn user_font_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
        .join("Microsoft")
        .join("Windows")
        .join("Fonts")
}

#[cfg(target_os = "macos")]
pub fn user_font_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Fonts")
}
