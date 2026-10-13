use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// 调用方分配的 JSON 输出缓冲区大小（与桥接层 calldyn.c 保持一致）。
pub const JSON_OUT_BUF_SIZE: usize = 65536;

/// 把 JSON 写进调用方提供的缓冲区，返回 0 成功、-1 失败。
fn write_json_out(json_out: *mut c_char, value: &Value) -> c_int {
    if json_out.is_null() {
        return -1;
    }
    let out_str = value.to_string();
    let bytes = out_str.as_bytes();
    if bytes.len() + 1 > JSON_OUT_BUF_SIZE {
        // 超长，写一个错误 JSON 而不是截断
        let fallback = json!({
            "code": 500900,
            "data": {},
            "msg": "json output exceeds buffer size"
        })
        .to_string();
        let fb = fallback.as_bytes();
        if fb.len() + 1 > JSON_OUT_BUF_SIZE {
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(fb.as_ptr(), json_out as *mut u8, fb.len());
            *json_out.add(fb.len()) = 0;
        }
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), json_out as *mut u8, bytes.len());
        *json_out.add(bytes.len()) = 0;
    }
    0
}

/// 构造错误 JSON 并写进 json_out。
pub fn write_error_json(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    write_json_out(
        json_out,
        &json!({
            "code": code,
            "data": {},
            "msg": msg
        }),
    )
}

/// 构造成功 JSON 并写进 json_out。
pub fn write_success_json(json_out: *mut c_char, msg: &str, data: Value) -> c_int {
    write_json_out(
        json_out,
        &json!({
            "code": 200,
            "data": data,
            "msg": msg
        }),
    )
}

/// 从 C 字符串解析 JSON。
pub fn parse_json<T>(json_in: *const c_char) -> Result<T, String>
where
    T: DeserializeOwned,
{
    if json_in.is_null() {
        return Err("json_in is null".into());
    }
    let body = unsafe { CStr::from_ptr(json_in) }
        .to_str()
        .map_err(|e| e.to_string())?;
    serde_json::from_str(body).map_err(|e| e.to_string())
}

// ============================================================
// 以下为测试辅助用的堆分配版本，生产路径不再使用。
// ============================================================

/// 将 serde_json::Value 序列化为堆分配的 C 字符串。
/// 仅供测试使用；生产代码走 `write_json_out` 写入调用方缓冲。
pub fn json_to_c_string(value: &Value) -> *mut c_char {
    let s = CString::new(value.to_string())
        .unwrap_or_else(|_| CString::new(r#"{"code":500,"msg":"json contains null"}"#).unwrap());
    s.into_raw()
}

/// 释放 `json_to_c_string` 返回的指针。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_json_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}
