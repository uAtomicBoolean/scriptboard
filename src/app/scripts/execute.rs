use crate::app::notifications::send_notification_with_custom_title;
use crate::{MainPageLogic, NotifStringEnum, NotifTypeEnum, Script, Scriptboard};
use slint::{ComponentHandle, Model, VecModel, Weak};

use super::commands;

/// Execute the script and store its output.
/// This function updates the scripts list to update the UI.
pub fn execute_script(ui_weak: Weak<Scriptboard>, script: Script, script_index: usize) {
    std::thread::spawn({
        move || {
            #[cfg(target_os = "linux")]
            let output = commands::linux::execute(&script);

            #[cfg(target_os = "macos")]
            let output = commands::macos::execute(&script);

            #[cfg(target_os = "windows")]
            let output = commands::windows::execute(&script);

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
