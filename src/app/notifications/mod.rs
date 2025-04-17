use crate::{NotifStringEnum, NotifTypeEnum, NotificationData, NotifsOverlayLogic, Scriptboard};
use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};

/// Send a notification to the UI.
/// This function uses a NotificationString for both the message and the title.
pub fn send_notification(
    ui: Weak<Scriptboard>,
    variant: NotifTypeEnum,
    title: NotifStringEnum,
    message: NotifStringEnum,
) {
    let ui = ui.upgrade().unwrap();
    let logic = ui.global::<NotifsOverlayLogic>();
    _send_notification(
        ui.as_weak(),
        NotificationData {
            variant: variant,
            title: logic.invoke_get_notif_string(title),
            message: logic.invoke_get_notif_string(message),
        },
    );
}

/// Send a notification to the UI with a custom title.
/// The title won't be translated as translation don't work in runtime.
pub fn send_notification_with_custom_title(
    ui: Weak<Scriptboard>,
    variant: NotifTypeEnum,
    title: SharedString,
    message: NotifStringEnum,
) {
    let ui = ui.upgrade().unwrap();
    let logic = ui.global::<NotifsOverlayLogic>();
    _send_notification(
        ui.as_weak(),
        NotificationData {
            variant: variant,
            title: title,
            message: logic.invoke_get_notif_string(message),
        },
    );
}

/// Private function that sends the notification to the UI.
fn _send_notification(ui: Weak<Scriptboard>, notif: NotificationData) {
    let ui = ui.upgrade().unwrap();
    let notifs_global = ui.global::<NotifsOverlayLogic>();

    let notifs_model = notifs_global.get_notifications();
    let notifs = notifs_model
        .as_any()
        .downcast_ref::<VecModel<NotificationData>>()
        .unwrap();

    notifs.insert(0, notif);
}
