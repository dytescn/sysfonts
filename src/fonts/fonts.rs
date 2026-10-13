use crate::utils::cjson::{parse_json, write_error_json, write_success_json};
use crate::utils::platform;
use serde::Deserialize;
use serde_json::json;
use std::ffi::c_char;
use std::os::raw::c_int;

// ============================================================
// 公共数据结构
// ============================================================

#[derive(Debug, serde::Serialize)]
pub struct FontFace {
    pub path: String,
    pub family: String,
    pub style: String,
    pub postscript_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InstallReq {
    font_path: String,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UninstallReq {
    postscript_name: String,
    #[serde(default)]
    delete_file: bool,
}

#[derive(Debug, Deserialize)]
struct ExportReq {
    postscript_name: String,
    dest_dir: String,
}

#[derive(Debug, Deserialize)]
struct ListReq {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    offset: Option<usize>,
}

// ============================================================
// 公共辅助
// ============================================================

/// 收集系统字体，支持分页。
///
/// `limit` 为 None 时返回全部；`offset` 为 None 时从 0 开始。
fn collect_all_faces(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<FontFace>, String> {
    use font_kit::source::SystemSource;

    let source = SystemSource::new();
    let families = source.all_families().map_err(|e| e.to_string())?;

    let skip = offset.unwrap_or(0);
    let take = limit.unwrap_or(usize::MAX);

    let mut faces: Vec<FontFace> = Vec::new();
    let mut seen = 0usize;

    for family in &families {
        if faces.len() >= take {
            break;
        }
        if let Ok(handles) = source.select_family_by_name(family) {
            for handle in handles.fonts() {
                if seen < skip {
                    seen += 1;
                    continue;
                }
                if faces.len() >= take {
                    break;
                }
                if let Ok(font) = handle.load() {
                    let path = match handle {
                        font_kit::handle::Handle::Path { path, .. } => {
                            path.to_string_lossy().to_string()
                        }
                        font_kit::handle::Handle::Memory { .. } => continue,
                    };
                    let postscript = font.postscript_name();
                    let props = font.properties();
                    faces.push(FontFace {
                        path,
                        family: family.clone(),
                        style: format!("{:?}", props.style),
                        postscript_name: postscript,
                    });
                }
            }
        }
    }
    Ok(faces)
}

fn handle_to_path(
    handle: font_kit::handle::Handle,
) -> Result<std::path::PathBuf, String> {
    match handle {
        font_kit::handle::Handle::Path { path, .. } => Ok(path),
        font_kit::handle::Handle::Memory { .. } => {
            Err("font is in memory, no file path".into())
        }
    }
}

// ============================================================
// FFI 导出接口：签名统一为 (json_in, json_out) -> c_int
// ============================================================

/// 安装字体。
/// 输入 JSON: `{"font_path": "...", "scope": "user"}`
#[unsafe(no_mangle)]
pub extern "C" fn post_install_font(
    json_in: *const c_char,
    json_out: *mut c_char,
) -> c_int {
    let req: InstallReq = match parse_json(json_in) {
        Ok(v) => v,
        Err(e) => return write_error_json(json_out, 400101, &e),
    };
    let scope = req.scope.as_deref().unwrap_or("user");
    match platform::install_font(&req.font_path, scope) {
        Ok(()) => write_success_json(json_out, "font installed", json!({})),
        Err(e) => write_error_json(json_out, 500101, &e),
    }
}

/// 获取系统字体列表。
///
/// 输入 JSON（可选）: `{"limit": 200, "offset": 0}`
/// json_in 为 null 时使用默认 limit=200，避免 JSON 超过调用方缓冲区。
#[unsafe(no_mangle)]
pub extern "C" fn post_list_system_fonts(
    json_in: *const c_char,
    json_out: *mut c_char,
) -> c_int {
    let req: ListReq = if json_in.is_null() {
        ListReq {
            limit: None,
            offset: None,
        }
    } else {
        match parse_json(json_in) {
            Ok(v) => v,
            Err(e) => return write_error_json(json_out, 400104, &e),
        }
    };

    // 默认 limit=200，避免超过 64KB 缓冲区
    let limit = req.limit.or(Some(200));

    match collect_all_faces(limit, req.offset) {
        Ok(faces) => write_success_json(json_out, "success", json!({ "fonts": faces })),
        Err(e) => write_error_json(json_out, 500200, &e),
    }
}

/// 卸载（可选删除文件）字体。
/// 输入 JSON: `{"postscript_name": "...", "delete_file": false}`
#[unsafe(no_mangle)]
pub extern "C" fn post_uninstall_font(
    json_in: *const c_char,
    json_out: *mut c_char,
) -> c_int {
    let req: UninstallReq = match parse_json(json_in) {
        Ok(v) => v,
        Err(e) => return write_error_json(json_out, 400102, &e),
    };
    match platform::uninstall_font(&req.postscript_name, req.delete_file) {
        Ok(()) => write_success_json(json_out, "font uninstalled", json!({})),
        Err(e) => write_error_json(json_out, 500102, &e),
    }
}

/// 导出字体：把系统已安装的字体复制到指定目录。
/// 输入 JSON: `{"postscript_name": "...", "dest_dir": "/tmp/out"}`
#[unsafe(no_mangle)]
pub extern "C" fn post_export_font(
    json_in: *const c_char,
    json_out: *mut c_char,
) -> c_int {
    let req: ExportReq = match parse_json(json_in) {
        Ok(v) => v,
        Err(e) => return write_error_json(json_out, 400103, &e),
    };

    use font_kit::source::SystemSource;

    let source = SystemSource::new();
    let handle = match source.select_by_postscript_name(&req.postscript_name) {
        Ok(h) => h,
        Err(e) => {
            return write_error_json(json_out, 500300, &format!("font not found: {e}"))
        }
    };

    let src_path = match handle_to_path(handle) {
        Ok(p) => p,
        Err(e) => return write_error_json(json_out, 500301, &e),
    };

    let filename = match src_path.file_name() {
        Some(n) => n.to_owned(),
        None => return write_error_json(json_out, 500302, "invalid font path"),
    };

    let dest_dir = std::path::PathBuf::from(&req.dest_dir);
    if let Err(e) = std::fs::create_dir_all(&dest_dir) {
        return write_error_json(json_out, 500303, &format!("create dest_dir failed: {e}"));
    }

    let dest = dest_dir.join(&filename);
    if let Err(e) = std::fs::copy(&src_path, &dest) {
        return write_error_json(json_out, 500304, &format!("copy failed: {e}"));
    }

    write_success_json(
        json_out,
        "font exported",
        json!({ "path": dest.to_string_lossy() }),
    )
}
