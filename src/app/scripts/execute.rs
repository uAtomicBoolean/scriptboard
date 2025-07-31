use crate::app::notifications::send_notification;
use crate::app::store::structs::StoredScript;
use crate::{MainPageLogic, NotifStringEnum, NotifTypeEnum, Script, Scriptboard};
use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::rc::Rc;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Execute the script and store its output.
/// This function updates the scripts list to update the UI.
pub fn execute_script(ui_weak: Weak<Scriptboard>, script: Script, script_index: usize) {
    std::thread::spawn({
        let light_script: StoredScript = script.into();
        move || {
            let parsed_args = match shlex::split(&light_script.args) {
                Some(pargs) => pargs,
                None => vec![String::new()],
            };

            #[cfg(target_os = "linux")]
            let mut cmd = get_linux_command(&light_script, parsed_args);

            #[cfg(target_os = "macos")]
            let mut cmd = get_macos_command(&light_script, parsed_args);

            #[cfg(target_os = "windows")]
            let mut cmd = get_windows_command(&light_script, parsed_args);

            let child_process = match cmd.spawn() {
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

            let process_stdout = match child_process.stdout {
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

            let rc_ui_weak = Rc::new(ui_weak.clone());
            let reader = BufReader::new(process_stdout);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| {
                    let index_clone = script_index.clone();
                    println!("{}", line.clone());
                    let _ = rc_ui_weak.upgrade_in_event_loop(move |ui| {
                        let scripts_model = ui.global::<MainPageLogic>().get_scripts();
                        let scripts = scripts_model
                            .as_any()
                            .downcast_ref::<VecModel<Script>>()
                            .unwrap();
                        let mut script = scripts.row_data(index_clone).unwrap();
                        let output_lines: VecModel<SharedString> =
                            script.output_lines.iter().collect();
                        output_lines.push(line.into());
                        script.output_lines = Rc::new(output_lines).into();
                        scripts.set_row_data(index_clone, script);
                    });
                });

            // Wait with output
            // Send send notification if enabled.

            // let mut script = script;
            // script.output = output.logs.into();
            // script.status_code = output.status_code;
            // script.running = false;
            // let _ = ui_weak.upgrade_in_event_loop(move |ui| {
            //     let scripts_model = ui.global::<MainPageLogic>().get_scripts();
            //     let scripts = scripts_model
            //         .as_any()
            //         .downcast_ref::<VecModel<Script>>()
            //         .unwrap();
            //     scripts.set_row_data(script_index, script.clone());
            //     if script.notifs_enabled {
            //         let (theme, message) = match output.status_code {
            //             0 => (NotifTypeEnum::Success, NotifStringEnum::ScriptExecutionDone),
            //             _ => (
            //                 NotifTypeEnum::Danger,
            //                 NotifStringEnum::ScriptExecutionFailed,
            //             ),
            //         };
            //         send_notification_with_custom_title(
            //             ui.as_weak(),
            //             theme,
            //             script.name.into(),
            //             message,
            //         );
            //     }
            // });
        }
    });
}

#[cfg(target_os = "linux")]
fn get_linux_command(script: &StoredScript, parsed_args: Vec<String>) -> Command {
    if !script.need_admin {
        use std::process::Stdio;

        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path)
            .args(parsed_args)
            .stdout(Stdio::piped());
        return cmd;
    }

    let mut cmd = Command::new("pkexec");
    cmd.args([&script.interpreter, &script.path])
        .args(parsed_args)
        .stdout(Stdio::piped());

    cmd
}

#[cfg(target_os = "macos")]
fn get_macos_command(script: &StoredScript, parsed_args: Vec<String>) -> Command {
    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path)
            .args(parsed_args)
            .stdout(Stdio::piped());
        return cmd;
    }

    let osascript_arg = format!(
        "do shell script \"{} {} {}\" with prompt \"{}\" with administrator privileges",
        &script.interpreter,
        &script.path,
        parsed_args.join(" "),
        &script.name,
    );

    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(osascript_arg).stdout(Stdio::piped());

    cmd
}

#[cfg(target_os = "windows")]
fn get_windows_command(script: &StoredScript, parsed_args: Vec<String>) -> Command {
    // Ask for admin password
    // Start-Process cmd -ArgumentList "/C your_command_here" -Verb RunAs
    // Example with custom interpreter :
    // Start-Process cmd -ArgumentList "/C python -c \"print('Hello, World!')\"" -Verb RunAs

    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg("-File")
            .arg(&script.path)
            .args(parsed_args)
            .creation_flags(0x08000000)
            .stdout(Stdio::piped());
        return cmd;
    }

    let script_args = format!(
        "\"/C {} {} {}\"",
        &script.interpreter,
        &script.path,
        parsed_args.join(" ")
    );

    let mut cmd: Command = Command::new("Start-Process");
    cmd.arg("cmd")
        .arg("-ArgumentList")
        .arg(script_args)
        .creation_flags(0x08000000)
        .stdout(Stdio::piped());

    cmd
}
