pub mod cjson;

#[cfg(windows)]
#[path = "platform_windows.rs"]
pub mod platform;

#[cfg(target_os = "macos")]
#[path = "platform_macos.rs"]
pub mod platform;

#[cfg(not(any(windows, target_os = "macos")))]
#[path = "platform_unix.rs"]
pub mod platform;
