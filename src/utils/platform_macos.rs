use core_foundation::base::TCFType;
use core_foundation::url::CFURL;
use core_text::font_manager;
use std::path::PathBuf;

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

    let dest_dir = if scope == "system" {
        PathBuf::from("/Library/Fonts")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Fonts")
    };
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(&filename);
    std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;

    let url = CFURL::from_path(&dest, false).ok_or("CFURL::from_path failed")?;
    font_manager::CTFontManagerRegisterFontsForURL(
        url.as_concrete_TypeRef(),
        font_manager::kCTFontManagerScopeProcess,
        std::ptr::null_mut(),
    );
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

    if delete_file {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
