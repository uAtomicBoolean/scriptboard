#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub struct ScriptOutput {
    pub logs: String,
    pub status_code: i32,
}
