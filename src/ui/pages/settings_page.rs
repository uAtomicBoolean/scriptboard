use crate::app::store;
use crate::{Scriptboard, SettingsPageLogic, UAppTheme, USelectOption};
use log::error;
use slint::{ComponentHandle, Model, Weak};

pub fn init_ui(ui: Weak<Scriptboard>) {
    let ui = ui.upgrade().unwrap();

    let logic = ui.global::<SettingsPageLogic>();

    logic.on_get_default_interpreter(|| store::get_default_interpreter().into());
    logic.on_get_notif_timeout(|| store::get_notif_timeout());
    logic.on_get_timeout_enabled(|| store::get_timeout_enabled());
    logic.on_get_scale_factor_current_index(|options| {
        let scale_factor = store::get_scale_factor();
        let sf_options: Vec<USelectOption> = options.iter().collect();
        // Unwrapping without checking as we know the content of sf_options hence the function shouldn't fail.
        sf_options
            .iter()
            .position(|o| o.value.parse::<f32>().unwrap() == scale_factor)
            .unwrap()
            .try_into()
            .unwrap()
    });
    logic.on_get_language_current_index(|options| {
        let language = store::get_language();
        let lg_options: Vec<USelectOption> = options.iter().collect();
        lg_options
            .iter()
            .position(|o| o.value == language)
            .unwrap()
            .try_into()
            .unwrap()
    });

    logic.on_save_settings({
        let ui_weak = ui.as_weak();
        move |def_inter, lang, sf, timeout, tt_enabled| {
            store::set_default_interpreter(def_inter.into());
            store::set_notif_timeout(timeout);
            store::set_timeout_enabled(tt_enabled);
            store::set_scale_factor(sf);
            store::set_language(lang.clone().into());

            let ui = ui_weak.upgrade().unwrap();
            let app_theme = ui.global::<UAppTheme>();
            app_theme.set_scale_factor(sf);

            if let Err(err) = slint::select_bundled_translation(lang.to_string().as_str()) {
                error!("Couldn't select the bundled translation.");
                error!("{}", err.to_string());
                std::process::exit(1);
            }

            // TODO send notification if error while saving.
        }
    });
}
