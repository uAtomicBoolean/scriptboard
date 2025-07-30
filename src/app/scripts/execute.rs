use crate::app::notifications::send_notification_with_custom_title;
use crate::{MainPageLogic, NotifStringEnum, NotifTypeEnum, Script, Scriptboard};
use slint::{ComponentHandle, Model, VecModel, Weak};
use std::os::windows::process::CommandExt;
use std::process::Command;

pub struct ScriptOutput {
    pub logs: String,
    pub status_code: i32,
}

/// Execute the script and store its output.
/// This function updates the scripts list to update the UI.
pub fn execute_script(ui_weak: Weak<Scriptboard>, script: Script, script_index: usize) {
    std::thread::spawn({
        move || {
            let parsed_args = match shlex::split(&script.args) {
                Some(pargs) => pargs,
                None => vec![String::new()],
            };

            #[cfg(target_os = "linux")]
            let mut cmd = get_linux_command(&script, parsed_args);

            #[cfg(target_os = "macos")]
            let mut cmd = get_macos_command(&script, parsed_args);

            #[cfg(target_os = "windows")]
            let mut cmd = get_windows_command(&script, parsed_args);

            let output = match cmd.output() {
                Ok(out) => ScriptOutput {
                    logs: String::from_utf8_lossy(&out.stdout).to_string(),
                    status_code: out.status.code().unwrap_or_default(),
                },
                Err(err) => ScriptOutput {
                    logs: err.to_string(),
                    status_code: 1,
                },
            };

            let mut script = script;
            script.output = output.logs.into();
            script.status_code = output.status_code;
            script.running = false;

            let _ = ui_weak.upgrade_in_event_loop(move |ui| {
                let scripts_model = ui.global::<MainPageLogic>().get_scripts();
                let scripts = scripts_model
                    .as_any()
                    .downcast_ref::<VecModel<Script>>()
                    .unwrap();

                scripts.set_row_data(script_index, script.clone());

                if script.notifs_enabled {
                    let (theme, message) = match output.status_code {
                        0 => (NotifTypeEnum::Success, NotifStringEnum::ScriptExecutionDone),
                        _ => (
                            NotifTypeEnum::Danger,
                            NotifStringEnum::ScriptExecutionFailed,
                        ),
                    };
                    send_notification_with_custom_title(
                        ui.as_weak(),
                        theme,
                        script.name.into(),
                        message,
                    );
                }
            });
        }
    });
}

#[cfg(target_os = "linux")]
fn get_linux_command(script: &Script, parsed_args: Vec<String>) -> Command {
    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path).args(parsed_args);
        return cmd;
    }

    let mut cmd = Command::new("pkexec");
    cmd.args([&script.interpreter, &script.path])
        .args(parsed_args);

    cmd
}

#[cfg(target_os = "macos")]
fn get_macos_command(script: &Script, parsed_args: Vec<String>) -> Command {
    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path).args(parsed_args);
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
    cmd.arg("-e").arg(osascript_arg);

    cmd
}

#[cfg(target_os = "windows")]
fn get_windows_command(script: &Script, parsed_args: Vec<String>) -> Command {
    // Ask for admin password
    // Start-Process cmd -ArgumentList "/C your_command_here" -Verb RunAs
    // Example with custom interpreter :
    // Start-Process cmd -ArgumentList "/C python -c \"print('Hello, World!')\"" -Verb RunAs

    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg("-File")
            .arg(&script.path)
            .args(parsed_args)
            .creation_flags(0x08000000);
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
        .creation_flags(0x08000000);

    cmd
}
