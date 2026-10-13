```rust
//! # sysfonts
//!
//! 跨平台系统字体管理器，通过 C FFI 暴露四个接口：安装、列表、卸载、导出。
//!
//! ## 快速开始
//!
//! ```no_run
//! use sysfonts::fonts::fonts::post_install_font;
//! use std::ffi::CString;
//!
//! let json_in = CString::new(r#"{"font_path":"/path/to/font.ttf","scope":"user"}"#).unwrap();
//! let mut buf = vec![0u8; 65536];
//! let rc = post_install_font(json_in.as_ptr(), buf.as_mut_ptr() as *mut _);
//! assert_eq!(rc, 0);
//! ```
//!
//! ## 接口约定
//!
//! - 所有导出函数签名为 `(const char* json_in, char* json_out) -> int`
//! - `json_out` 缓冲区由调用方分配，至少 65536 字节
//! - 返回 0 表示 FFI 层成功，业务状态看 `json_out` 里的 `code` 字段
//! - 输入 JSON 由调用方拥有，函数不释放它
//!
//! ## 模块组织
//!
//! - [`fonts`]：四个 FFI 导出接口
//! - [`utils::cjson`]：JSON 编解码与缓冲区写入
//! - [`utils::platform`]：平台相关的安装/卸载实现

pub mod fonts;
pub mod utils;
