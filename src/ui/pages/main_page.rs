use crate::app::notifications::send_notification;
use crate::app::store;
use crate::{
    MainPageLogic, NotifStringEnum, NotifTypeEnum, Script, ScriptOutputWindow, Scriptboard,
    UAppTheme,
};
use log::{error, warn};
use slint::{ComponentHandle, Model, ModelRc, VecModel, Weak};
use std::rc::Rc;

pub fn init_ui(ui: Weak<Scriptboard>, scripts: Rc<VecModel<Script>>) {
    let ui = ui.upgrade().unwrap();

    let logic = ui.global::<MainPageLogic>();

    logic.set_scripts(ModelRc::new(scripts.clone()));

    logic.on_execute_script({
        let ui_weak = ui.as_weak();
        let scripts_clone = scripts.clone();
        move |script_index| {
            crate::app::scripts::execute::execute_script(
                ui_weak.clone(),
                scripts_clone.clone(),
                script_index as usize,
            );
        }
    });

    logic.on_open_script_output({
        let ui_weak = ui.as_weak();
        let scripts_clone: Rc<VecModel<Script>> = scripts.clone();
        move |index| {
            let script: Script = scripts_clone.row_data(index as usize).unwrap();

            let output_window = match ScriptOutputWindow::new() {
                Ok(uw) => uw,
                Err(err) => {
                    warn!(
                        "Couldn't create the output window instance for script \"{}\".",
                        script.name
                    );
                    warn!("{}", err.to_string());
                    send_notification(
                        ui_weak.clone(),
                        NotifTypeEnum::Danger,
                        NotifStringEnum::ErrorOpenOutputWindowTitle,
                        NotifStringEnum::ErrorOpenOutputWindowMessage,
                    );
                    return;
                }
            };
            let script_name = script.name.clone();
            output_window.set_scripts(ModelRc::new(scripts_clone.clone()));
            output_window.set_script_index(index);

            let app_theme = output_window.global::<UAppTheme>();
            app_theme.set_scale_factor(store::get_scale_factor());

            // As global are not shared between components, we cannot use
            // MainPageLogic.execute-script() in ScriptOutputWindow.
            // Hence we need to manage a new callback from this window.
            output_window.on_execute_script({
                let ui_weak = ui_weak.clone();
                let scripts_clone = scripts.clone();
                move || {
                    crate::app::scripts::execute::execute_script(
                        ui_weak.clone(),
                        scripts_clone.clone(),
                        index as usize,
                    );
                }
            });

            output_window.on_update_preserve_output({
                let scripts_clone = scripts.clone();
                move |preserve_output| {
                    let mut script = scripts_clone.row_data(index as usize).unwrap();
                    script.preserve_output = preserve_output;
                    scripts_clone.set_row_data(index as usize, script);
                }
            });

            if let Err(err) = slint::select_bundled_translation(&store::get_language()) {
                error!("Couldn't select the bundled translation.");
                error!("{}", err.to_string());
                std::process::exit(1);
            }

            let run_result = output_window.show();

            if let Err(err) = run_result {
                // Not sending a notification because some errors are not critical.
                // Example: "Nested event loops are not supported" is raised on debian 11 wayland.
                warn!(
                    "Couldn't open the output window for script \"{}\".",
                    script_name
                );
                warn!("{}", err.to_string());
            }
        }
    });
}
