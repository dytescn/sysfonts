//! 四个 FFI 导出接口的集成测试。

mod common;

use common::*;
use sysfonts::fonts::fonts::{
    post_export_font, post_install_font, post_list_system_fonts, post_uninstall_font,
};

// ============================================================
// post_install_font
// ============================================================

#[test]
fn test_install_invalid_json() {
    let raw = call_ffi(Some("not json"), post_install_font);
    let v = parse_and_assert_code(&raw, 400101);
    assert!(v["msg"].as_str().unwrap().contains("expected"));
}

#[test]
fn test_install_missing_field() {
    let raw = call_ffi(Some(r#"{"scope":"user"}"#), post_install_font);
    parse_and_assert_code(&raw, 400101);
}

#[test]
fn test_install_nonexistent_file() {
    let raw = call_ffi(
        Some(r#"{"font_path":"/no/such/font.ttf","scope":"user"}"#),
        post_install_font,
    );
    let v = parse_and_assert_code(&raw, 500101);
    assert!(v["msg"].as_str().unwrap().contains("not found"));
}

#[test]
fn test_install_default_scope_is_user() {
    let raw = call_ffi(
        Some(r#"{"font_path":"/no/such/font.ttf"}"#),
        post_install_font,
    );
    parse_and_assert_code(&raw, 500101);
}

// ============================================================
// post_list_system_fonts
// ============================================================

#[test]
fn test_list_returns_success() {
    let raw = call_ffi_no_arg(post_list_system_fonts);
    let v = parse_and_assert_code(&raw, 200);
    let fonts = v["data"]["fonts"]
        .as_array()
        .expect("data.fonts should be an array");
    assert!(!fonts.is_empty(), "system should have at least one font");
}

#[test]
fn test_list_font_face_shape() {
    let v = do_list();
    let first = &v["data"]["fonts"][0];
    assert!(first["path"].is_string());
    assert!(first["family"].is_string());
    assert!(first["style"].is_string());
    assert!(first.get("postscript_name").is_some());
}

// ============================================================
// post_uninstall_font
// ============================================================

#[test]
fn test_uninstall_invalid_json() {
    let raw = call_ffi(Some("not json"), post_uninstall_font);
    parse_and_assert_code(&raw, 400102);
}

#[test]
fn test_uninstall_missing_postscript_name() {
    let raw = call_ffi(Some(r#"{"delete_file":false}"#), post_uninstall_font);
    parse_and_assert_code(&raw, 400102);
}

#[test]
fn test_uninstall_nonexistent_font() {
    let raw = call_ffi(
        Some(r#"{"postscript_name":"ThisFontDoesNotExist-XXXX","delete_file":false}"#),
        post_uninstall_font,
    );
    parse_and_assert_code(&raw, 500102);
}

// ============================================================
// post_export_font
// ============================================================

#[test]
fn test_export_invalid_json() {
    let raw = call_ffi(Some("not json"), post_export_font);
    parse_and_assert_code(&raw, 400103);
}

#[test]
fn test_export_nonexistent_font() {
    let tmp = temp_dir("export_err");
    let req = serde_json::json!({
        "postscript_name": "ThisFontDoesNotExist-XXXX",
        "dest_dir": tmp.to_string_lossy(),
    })
    .to_string();
    let raw = call_ffi(Some(&req), post_export_font);
    parse_and_assert_code(&raw, 500300);
    let _ = std::fs::remove_dir_all(&tmp);
}
