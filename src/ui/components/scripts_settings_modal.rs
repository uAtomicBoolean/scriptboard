use crate::app::store;
use crate::{Script, ScriptSettingsModalLogic, Scriptboard};
use slint::{ComponentHandle, Model, VecModel, Weak};
use std::rc::Rc;

pub fn init_ui(ui: Weak<Scriptboard>, scripts: Rc<VecModel<Script>>) {
    let ui = ui.upgrade().unwrap();

    let logic = ui.global::<ScriptSettingsModalLogic>();

    logic.on_save_script_settings({
        let scripts_model = scripts.clone();
        move |id, name, interpreter, args, admin, notif| {
            let mut script = scripts_model.iter().find(|s| s.id == id).unwrap();
            script.name = name;
            script.interpreter = interpreter;
            script.args = args;
            script.need_admin = admin;
            script.notifs_enabled = notif;
            let script_pos = scripts_model.iter().position(|s| s.id == id).unwrap();
            scripts_model.set_row_data(script_pos, script.clone());
            store::update_script(script_pos, script);
        }
    });

    logic.on_delete_script({
        let scripts_model = scripts.clone();
        move |script_index| {
            scripts_model.remove(script_index as usize);
            store::remove_script(script_index);
        }
    });
}
