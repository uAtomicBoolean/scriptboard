use super::commands::*;
use crate::app::notifications::{send_notification, send_notification_with_custom_title};
use crate::app::store::structs::StoredScript;
use crate::{MainPageLogic, NotifStringEnum, NotifTypeEnum, Script, Scriptboard};
use slint::{ComponentHandle, Model, VecModel, Weak};
use std::io::{BufReader, Read};
use std::process::Output;
use std::rc::Rc;

/// Prepare the scripts before its execution then start the execution.
pub fn execute_script(
    ui_weak: Weak<Scriptboard>,
    scripts: Rc<VecModel<Script>>,
    script_index: usize,
) {
    let mut script = scripts.row_data(script_index as usize).unwrap();
    script.running = true;
    if !script.preserve_output {
        script.output = "".into();
    } else {
        script.output.push_str("\n");
    }
    scripts.set_row_data(script_index as usize, script.clone());
    run_execution_script(ui_weak.clone(), script, script_index as usize);
}

/// Execute the script and store its output.
/// This function updates the scripts list to update the UI.
fn run_execution_script(ui_weak: Weak<Scriptboard>, script: Script, script_index: usize) {
    std::thread::spawn({
        let light_script: StoredScript = script.into();
        move || {
            let parsed_args = match shlex::split(&light_script.args) {
                Some(pargs) => pargs,
                None => vec![String::new()],
            };

            // Build and execute the command as a child process.
            #[cfg(target_os = "linux")]
            let mut cmd = get_linux_command(&light_script, parsed_args);

            #[cfg(target_os = "macos")]
            let mut cmd = get_macos_command(&light_script, parsed_args);

            #[cfg(target_os = "windows")]
            let mut cmd = get_windows_command(&light_script, parsed_args);

            let mut child_process = match cmd.spawn() {
                Ok(cp) => cp,
                Err(err) => {
                    log::error!("Error while spawning the script {}", &light_script.name);
                    log::error!("{}", err.to_string());
                    send_notification(
                        ui_weak.clone(),
                        NotifTypeEnum::Danger,
                        NotifStringEnum::ErrorSpawnScript,
                        NotifStringEnum::CheckLogs,
                    );
                    return;
                }
            };

            // Get the child stdout without partially borrowing the child process.
            // This allows us to wait for the child's output at the end.
            let process_stdout = match child_process.stdout.take() {
                Some(stdout) => stdout,
                None => {
                    log::error!("No output found for script {}", &light_script.name);
                    send_notification(
                        ui_weak.clone(),
                        NotifTypeEnum::Danger,
                        NotifStringEnum::ErrorScriptOutput,
                        NotifStringEnum::ScriptExecutionCancelled,
                    );
                    return;
                }
            };

            // Reading the child's output line by line to live-update the output window.
            let reader = BufReader::new(process_stdout);
            let uiw_rc = Rc::new(ui_weak.clone());

            let mut buffer = vec![];
            for byte_result in reader.bytes() {
                let byte = match byte_result {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                if byte == b'\n' {
                    let line = String::from_utf8_lossy(&buffer).to_string();
                    let _ = uiw_rc.upgrade_in_event_loop(move |ui| {
                        update_script_output(ui, line, script_index)
                    });
                    buffer.clear()
                }
                buffer.push(byte);
            }

            // Waiting for the child's status code via its output.
            let output = match child_process.wait_with_output() {
                Ok(out) => Some(out),
                Err(err) => {
                    log::error!("Could not get output after waiting for script end.");
                    log::error!("{}", err.to_string());
                    send_notification(
                        ui_weak.clone(),
                        NotifTypeEnum::Danger,
                        NotifStringEnum::ErrorScriptOutput,
                        NotifStringEnum::ScriptExecutionCancelled,
                    );
                    None
                }
            };

            let _ = ui_weak
                .upgrade_in_event_loop(move |ui| finish_script_execution(ui, output, script_index));
        }
    });
}

// Add a new line to the script's output.
fn update_script_output(ui: Scriptboard, line: String, script_index: usize) {
    let scripts_model = ui.global::<MainPageLogic>().get_scripts();
    let scripts = scripts_model
        .as_any()
        .downcast_ref::<VecModel<Script>>()
        .unwrap();

    let mut script = scripts.row_data(script_index).unwrap();
    script.output.push_str(&line);

    scripts.set_row_data(script_index, script);
}

// Manage the last part of the script's execution by checking its status code and sending the notifications.
fn finish_script_execution(ui: Scriptboard, output: Option<Output>, script_index: usize) {
    let scripts_model = ui.global::<MainPageLogic>().get_scripts();
    let scripts = scripts_model
        .as_any()
        .downcast_ref::<VecModel<Script>>()
        .unwrap();

    let mut script = scripts.row_data(script_index).unwrap();
    script.running = false;

    if output.is_none() {
        script.status_code = -1;
        scripts.set_row_data(script_index, script.clone());
        return;
    }

    let output = output.unwrap();
    let (theme, message) = match output.status.code() {
        Some(code) if code == 0 => {
            script.status_code = code;
            (NotifTypeEnum::Success, NotifStringEnum::ScriptExecutionDone)
        }
        _ => (
            NotifTypeEnum::Danger,
            NotifStringEnum::ScriptExecutionFailed,
        ),
    };
    scripts.set_row_data(script_index, script.clone());

    if script.notifs_enabled {
        send_notification_with_custom_title(ui.as_weak(), theme, script.name.into(), message);
    }
}
