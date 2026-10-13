use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{AddFontResourceW, RemoveFontResourceW};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageW, HWND_BROADCAST, WM_FONTCHANGE,
};

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn font_dest_path(filename: &str, scope: &str) -> PathBuf {
    if scope == "system" {
        PathBuf::from(std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into()))
            .join("Fonts")
            .join(filename)
    } else {
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_default())
            .join("Microsoft")
            .join("Windows")
            .join("Fonts")
            .join(filename)
    }
}

fn write_registry(font_name: &str, value: &str, scope: &str) -> Result<(), String> {
    let root: HKEY = if scope == "system" {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let subkey: Vec<u16> = to_wide(r"Software\Microsoft\Windows NT\CurrentVersion\Fonts");
    let mut hkey = HKEY::default();
    let status = unsafe {
        RegCreateKeyExW(
            root,
            PCWSTR(subkey.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
    };
    if status.is_err() {
        return Err(format!("RegCreateKeyExW failed: {:?}", status));
    }

    let value_name: Vec<u16> = to_wide(font_name);
    let value_data: Vec<u16> = to_wide(value);
    let data_bytes = std::slice::from_raw_parts(
        value_data.as_ptr() as *const u8,
        value_data.len() * 2,
    );
    let set = unsafe {
        RegSetValueExW(hkey, PCWSTR(value_name.as_ptr()), None, REG_SZ, Some(data_bytes))
    };
    unsafe {
        let _ = RegCloseKey(hkey);
    }
    if set.is_err() {
        return Err(format!("RegSetValueExW failed: {:?}", set));
    }
    Ok(())
}

pub fn install_font(font_path: &str, scope: &str) -> Result<(), String> {
    let src = PathBuf::from(font_path);
    if !src.exists() {
        return Err(format!("font file not found: {}", font_path));
    }
    let filename = src
        .file_name()
        .ok_or("invalid font path")?
        .to_string_lossy()
        .to_string();

    let dest = font_dest_path(&filename, scope);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;

    let wide = to_wide(dest.to_string_lossy().as_ref());
    let ret = unsafe { AddFontResourceW(PCWSTR(wide.as_ptr())) };
    if ret == 0 {
        let _ = std::fs::remove_file(&dest);
        return Err("AddFontResourceW failed".into());
    }

    let _ = write_registry(&filename, &dest.to_string_lossy(), scope);

    unsafe {
        SendMessageW(HWND_BROADCAST, WM_FONTCHANGE, None, None);
    }
    Ok(())
}

pub fn uninstall_font(postscript_name: &str, delete_file: bool) -> Result<(), String> {
    use font_kit::source::SystemSource;
    let source = SystemSource::new();
    let handle = source
        .select_by_postscript_name(postscript_name)
        .map_err(|e| format!("font not found: {e}"))?;
    let path = handle
        .load()
        .map_err(|e| e.to_string())?
        .source()
        .to_string();

    let wide = to_wide(&path);
    unsafe {
        RemoveFontResourceW(PCWSTR(wide.as_ptr()));
        SendMessageW(HWND_BROADCAST, WM_FONTCHANGE, None, None);
    }

    if delete_file {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
