use crate::{NotificationData, NotifsOverlayLogic, Scriptboard, app::store};
use slint::{ComponentHandle, Model, VecModel, Weak};

pub fn init_ui(ui: Weak<Scriptboard>) {
    let ui = ui.upgrade().unwrap();
    let logic = ui.global::<NotifsOverlayLogic>();

    logic.on_get_timeout(|| (store::get_notif_timeout() * 1000).into());
    logic.on_get_timeout_enabled(|| store::get_timeout_enabled());

    logic.on_remove_notification({
        let notifs = logic.get_notifications();
        move |script_index| {
            let notifs = notifs
                .as_any()
                .downcast_ref::<VecModel<NotificationData>>()
                .unwrap();
            notifs.remove(script_index as usize);
        }
    });
}
