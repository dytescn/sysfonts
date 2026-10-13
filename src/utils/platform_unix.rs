use std::path::PathBuf;
use std::process::Command;

pub fn install_font(font_path: &str, scope: &str) -> Result<(), String> {
    let src = PathBuf::from(font_path);
    if !src.exists() {
        return Err(format!("font file not found: {}", font_path));
    }
    let filename = src.file_name().ok_or("invalid font path")?;

    let dest_dir = if scope == "system" {
        PathBuf::from("/usr/local/share/fonts")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share/fonts")
    };
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let dest = dest_dir.join(filename);
    std::fs::copy(&src, &dest).map_err(|e| e.to_string())?;

    Command::new("fc-cache")
        .arg("-f")
        .arg(&dest_dir)
        .output()
        .map_err(|e| format!("fc-cache failed: {e}"))?;
    Ok(())
}

pub fn uninstall_font(postscript_name: &str, delete_file: bool) -> Result<(), String> {
    use font_kit::source::SystemSource;
    let source = SystemSource::new();
    let handle = source
        .select_by_postscript_name(postscript_name)
        .map_err(|e| format!("font not found: {e}"))?;

    let path = match handle {
    font_kit::handle::Handle::Path { path, .. } => path.to_string_lossy().to_string(),
    font_kit::handle::Handle::Memory { .. } => {
        return Err("font is loaded from memory, cannot locate file".into())
    }
};
    if delete_file {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }

    let _ = Command::new("fc-cache").arg("-f").output();
    Ok(())
}
