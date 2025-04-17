use crate::app::{scripts, store};
use crate::{AddScriptModalLogic, Script, ScriptSelectionResult, Scriptboard};
use slint::{ComponentHandle, SharedString, VecModel, Weak};
use std::rc::Rc;

pub fn init_ui(ui: Weak<Scriptboard>, scripts: Rc<VecModel<Script>>) {
    let ui = ui.upgrade().unwrap();

    let logic = ui.global::<AddScriptModalLogic>();

    logic.on_get_default_interpreter(|| store::get_default_interpreter().into());

    logic.on_select_script_file(move || {
        let selected_script = scripts::select_script();
        if let Some((name, path)) = selected_script {
            return ScriptSelectionResult {
                script_selected: true,
                filename: name.into(),
                path: path.into(),
            };
        }

        ScriptSelectionResult {
            script_selected: false,
            filename: SharedString::new(),
            path: SharedString::new(),
        }
    });

    logic.on_load_script({
        let scripts_model = scripts.clone();
        move |mut script| {
            script.id = store::add_script(script.clone()).into();
            scripts_model.push(script);
        }
    });
}
