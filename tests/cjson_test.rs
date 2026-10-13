//! cjson 模块测试。

use serde_json::json;
use sysfonts::utils::cjson::{
    json_to_c_string, parse_json, write_error_json, write_success_json, JSON_OUT_BUF_SIZE,
};
use std::ffi::{CStr, CString};
use sysfonts::utils::cjson::free_json_string;

#[test]
fn test_write_error_json_into_buffer() {
    let mut buf = vec![0u8; JSON_OUT_BUF_SIZE];
    let rc = write_error_json(buf.as_mut_ptr() as *mut _, 400101, "bad input");
    assert_eq!(rc, 0);
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) }
        .to_string_lossy()
        .into_owned();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["code"], 400101);
    assert_eq!(v["msg"], "bad input");
    assert_eq!(v["data"], json!({}));
}

#[test]
fn test_write_success_json_into_buffer() {
    let mut buf = vec![0u8; JSON_OUT_BUF_SIZE];
    let rc = write_success_json(buf.as_mut_ptr() as *mut _, "ok", json!({"k": "v"}));
    assert_eq!(rc, 0);
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) }
        .to_string_lossy()
        .into_owned();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["k"], "v");
}

#[test]
fn test_write_json_out_null_returns_error() {
    let rc = write_error_json(std::ptr::null_mut(), 1, "x");
    assert_eq!(rc, -1);
}

#[test]
fn test_parse_json_valid() {
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Req { name: String }
    let c = CString::new(r#"{"name":"hello"}"#).unwrap();
    let r: Req = parse_json(c.as_ptr()).unwrap();
    assert_eq!(r, Req { name: "hello".into() });
}

#[test]
fn test_parse_json_invalid() {
    let c = CString::new("not json").unwrap();
    let r: Result<serde_json::Value, _> = parse_json(c.as_ptr());
    assert!(r.is_err());
}

#[test]
fn test_parse_json_null() {
    let r: Result<serde_json::Value, _> = parse_json(std::ptr::null());
    assert!(r.is_err());
}

#[test]
fn test_json_to_c_string_roundtrip() {
    let v = json!({"a": 1, "b": "two"});
    let ptr = json_to_c_string(&v);
    assert!(!ptr.is_null());
    let s = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
    unsafe { free_json_string(ptr) };
    let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, v);
}
