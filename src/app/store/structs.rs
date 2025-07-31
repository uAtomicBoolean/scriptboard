use crate::Script;
use serde::{Deserialize, Serialize};
use slint::SharedString;

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub default_interpreter: String,
    pub notif_timeout: i32,
    pub timeout_enabled: bool,
    pub scale_factor: f32,
    pub language: String,
    pub scripts: Vec<StoredScript>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredScript {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub path: String,
    pub need_admin: bool,
    pub notifs_enabled: bool,
    pub interpreter: String,
    pub args: String,
}

impl From<Script> for StoredScript {
    fn from(script: Script) -> Self {
        StoredScript {
            id: script.id.into(),
            name: script.name.into(),
            filename: script.filename.into(),
            path: script.path.into(),
            need_admin: script.need_admin,
            notifs_enabled: script.notifs_enabled,
            interpreter: script.interpreter.into(),
            args: script.args.into(),
        }
    }
}

impl Into<Script> for StoredScript {
    fn into(self) -> Script {
        Script {
            id: self.id.into(),
            name: self.name.into(),
            filename: self.filename.into(),
            path: self.path.into(),
            need_admin: self.need_admin,
            notifs_enabled: self.notifs_enabled,
            interpreter: self.interpreter.into(),
            args: self.args.into(),
            running: false,
            output: SharedString::new(),
            status_code: 0,
        }
    }
}

impl Into<Script> for &StoredScript {
    fn into(self) -> Script {
        Script {
            id: self.id.clone().into(),
            name: self.name.clone().into(),
            filename: self.filename.clone().into(),
            path: self.path.clone().into(),
            need_admin: self.need_admin,
            notifs_enabled: self.notifs_enabled,
            interpreter: self.interpreter.clone().into(),
            args: self.args.clone().into(),
            running: false,
            output: SharedString::new(),
            status_code: 0,
        }
    }
}
