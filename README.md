# sysfonts

跨平台系统字体管理器，通过 C FFI 暴露四个接口：安装、列表、卸载、导出。

## 特性

- **跨平台**：Windows（GDI + 注册表）、macOS（Core Text）、Linux（fontconfig）
- **C ABI**：所有接口以 JSON 字符串进出，签名统一为 `(const char* json_in, char* json_out) -> int`
- **线程安全**：每个接口无共享可变状态，可并发调用
- **错误码分段**：`400xxx` 客户端错误，`500xxx` 服务端错误

## 接口

| 符号 | 用途 | 输入 | 输出 |
|---|---|---|---|
| `post_install_font` | 安装字体 | `{"font_path": "...", "scope": "user"}` | `{"code":200,"data":{},"msg":"..."}` |
| `post_list_system_fonts` | 列出系统字体 | `{"limit": 200, "offset": 0}`（可选） | `{"code":200,"data":{"fonts":[...]},"msg":"..."}` |
| `post_uninstall_font` | 卸载字体 | `{"postscript_name": "...", "delete_file": false}` | `{"code":200,"data":{},"msg":"..."}` |
| `post_export_font` | 导出字体 | `{"postscript_name": "...", "dest_dir": "..."}` | `{"code":200,"data":{"path":"..."},"msg":"..."}` |

## 使用方式

`sysfonts` 有两种使用方式：

### 1. 作为 Rust 库直接调用

```rust
use sysfonts::fonts::fonts::post_install_font;
use std::ffi::CString;

let json_in = CString::new(r#"{"font_path":"/path/to/font.ttf","scope":"user"}"#).unwrap();
let mut buf = vec![0u8; 65536];
let rc = post_install_font(json_in.as_ptr(), buf.as_mut_ptr() as *mut _);
assert_eq!(rc, 0);

