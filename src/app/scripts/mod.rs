mod commands;
pub mod execute;

use rfd::FileDialog;

/// Open a dialog to select a script
/// All file types are currently accepted
pub fn select_script() -> Option<(String, String)> {
    match FileDialog::new().pick_file() {
        Some(file) => {
            let script_name = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let script_path = file.to_string_lossy().into_owned();
            Some((script_name, script_path))
        }
        None => None,
    }
}
