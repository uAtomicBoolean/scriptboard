// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_storage;

use std::error::Error;

use rfd::FileDialog;
use slint::{Model, SharedString};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let stored_scripts = file_storage::get_scripts();

    let ui = Scriptboard::new()?;

    // Updating the default ID in the UI.
    if stored_scripts.len() > 0 {
        let highest_id = stored_scripts.iter().max_by_key(|&s| s.id).unwrap().id;
        ui.set_script_id_incr(highest_id + 1);
    }

    // Load the stored scripts into the UI.
    let ui_scripts: Vec<Script> = stored_scripts
        .iter()
        .map(|s| Script {
            id: s.id,
            description: SharedString::from(&s.description),
            name: SharedString::from(&s.name),
            path: SharedString::from(&s.path),
            running: false,
            errored: false,
        })
        .collect();

    ui.set_scripts(std::rc::Rc::new(slint::VecModel::from(ui_scripts)).into());

    ui.on_script_select({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            script_select(ui);
        }
    });

    ui.on_add_script({
        let ui_handle = ui.as_weak();
        move |script_id, description, name, path| {
            let ui = ui_handle.unwrap();
            add_script(ui, script_id, description, name, path);
        }
    });

    ui.on_remove_script({
        let ui_handle = ui.as_weak();
        move |script_id| {
            let ui = ui_handle.unwrap();
            remove_script(ui, script_id);
        }
    });

    ui.on_execute_script({
        let ui_handle = ui.as_weak();
        move |id, path| {
            let ui_handle = ui_handle.clone();
            std::thread::spawn(move || {
                #[cfg(target_family = "unix")]
                let code = execute_script_unix(path);
                #[cfg(target_family = "windows")]
                let code = execute_script_windows(path);

                let _ = ui_handle.upgrade_in_event_loop(move |handle| {
                    let mut scripts: Vec<Script> = handle.get_scripts().iter().collect();

                    let script_index = scripts.iter().position(|s| s.id == id).unwrap();
                    scripts[script_index].running = false;
                    scripts[script_index].errored = code != 0;

                    handle.set_scripts(std::rc::Rc::new(slint::VecModel::from(scripts)).into());
                });
            });
        }
    });

    ui.run()?;

    Ok(())
}

/// Open a file dialog to select the new script to add.
///
/// # Example
/// ```
/// let ui = ui_handle.unwrap();
/// script_select(ui);
/// ```
fn script_select(ui: Scriptboard) {
    let file_option = FileDialog::new().set_title("Select a script").pick_file();
    if let Some(file) = file_option {
        ui.set_new_script_name(SharedString::from(
            file.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
        ));
        ui.set_new_script_path(SharedString::from(file.to_str().unwrap_or_default()));
        ui.set_missing_path(false);
    }
}

/// Add a script in the list and store it in a file.
/// The file is stored in the user's local data directory.
///
/// # Example
/// ```
/// let ui = ui_handle.unwrap();
/// add_script(ui, id, description, name, path);
/// ```
fn add_script(
    ui: Scriptboard,
    id: i32,
    description: SharedString,
    name: SharedString,
    path: SharedString,
) {
    if description.is_empty() || path.is_empty() {
        ui.set_missing_description(description.is_empty());
        ui.set_missing_path(path.is_empty());
        return;
    }

    let mut scripts: Vec<Script> = ui.get_scripts().iter().collect();

    scripts.push(Script {
        id: id,
        description: description.clone(),
        name: name.clone(),
        path: path.clone(),
        running: false,
        errored: false,
    });

    ui.set_scripts(std::rc::Rc::new(slint::VecModel::from(scripts)).into());

    ui.invoke_reset_input_zone();
    file_storage::store_script(file_storage::StoredScript {
        id: id,
        description: description.to_string(),
        name: name.to_string(),
        path: path.to_string(),
    });
}

/// Remove a script from the list and the file storage.
///
/// # Example
/// ```
/// let ui = ui_handle.unwrap();
/// remove_script(ui, id);
/// ```
fn remove_script(ui: Scriptboard, id: i32) {
    let scripts: Vec<Script> = ui.get_scripts().iter().collect();
    let filtered_scripts: Vec<Script> = scripts.into_iter().filter(|s| s.id != id).collect();

    ui.set_scripts(std::rc::Rc::new(slint::VecModel::from(filtered_scripts)).into());
    file_storage::remove_script(id);
}

#[cfg(target_family = "unix")]
/// Execute a script on an unix OS with its path.
///
/// # Example
/// ```
/// let script_path = SharedString::from("/usr/bin/clear");
/// execute_script(script_path);
/// ```
fn execute_script_unix(path: SharedString) -> i32 {
    let cmd = std::process::Command::new("sh").arg(path).output();
    match cmd {
        Ok(output) => {
            return output.status.code().unwrap_or_default();
        }
        Err(err) => {
            println!("Couldn't execute script");
            println!("{}", err.to_string());
            return -1;
        }
    }
}

#[cfg(target_family = "windows")]
/// Execute a script on windows with its path.
///
/// # Example
/// ```
/// let script_path = SharedString::from("/usr/bin/clear");
/// execute_script(script_path);
/// ```
fn execute_script_windows(path: SharedString) -> i32 {
    // TODO Execute script on windows.
    0
}
