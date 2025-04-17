use crate::{AboutPageLogic, Scriptboard};
use log::warn;
use slint::{ComponentHandle, Weak};

pub fn init_ui(ui: Weak<Scriptboard>) {
    let ui = ui.upgrade().unwrap();

    let logic = ui.global::<AboutPageLogic>();

    logic.on_open_url(|url| {
        if let Err(err) = open::that(&url) {
            warn!("Couldn't open URL \"{url}\".");
            warn!("{}", err.to_string());
        }
    });
}
