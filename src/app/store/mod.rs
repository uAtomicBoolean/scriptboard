pub mod structs;

use log::error;
use once_cell::sync::Lazy;
use std::{
    io::BufReader,
    sync::{Arc, Mutex, MutexGuard},
};
use structs::StoredScript;
use uuid::Uuid;

pub use self::structs::Store;
use crate::Script;

static STORE: Lazy<Arc<Mutex<Store>>> = Lazy::new(|| {
    let store_path = crate::app::utils::get_store_path();
    // Creating the file here even though the folder is created right before.
    // This reduce code repetition if the folder is already created but not the file.
    if !store_path.exists() {
        let store = Store {
            #[cfg(target_family = "unix")]
            default_interpreter: "sh".into(),
            #[cfg(target_family = "windows")]
            default_interpreter: "powershell.exe".into(),
            notif_timeout: 10,
            timeout_enabled: true,
            scale_factor: 1f32,
            language: "en".into(),
            scripts: vec![],
        };

        write_store(&&store);
        return Arc::new(Mutex::new(store));
    }

    let store_file = match std::fs::File::open(&store_path) {
        Ok(file) => file,
        Err(err) => {
            error!("Error while opening the json store.");
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    };

    match serde_json::from_reader(BufReader::new(store_file)) {
        Ok(store) => Arc::new(Mutex::new(store)),
        Err(err) => {
            error!("Error while reading the json store.");
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    }
});

fn get_store_guard() -> MutexGuard<'static, Store> {
    match STORE.lock() {
        Ok(guard) => guard,
        Err(err) => {
            error!("Store is poisoned.");
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    }
}

/// Write the current store in the json file.
pub fn write_store(store: &Store) {
    let json_store = match serde_json::to_string(store) {
        Ok(jt) => jt,
        Err(err) => {
            error!("Error while converting store to json string.");
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    };

    let store_path = crate::app::utils::get_store_path();
    if let Err(err) = std::fs::write(store_path, json_store) {
        error!("Couldn't write the store to its file.");
        error!("{}", err.to_string());
    }
}

/* -------------------------------------- */
/* Data saving and settings function      */
/* -------------------------------------- */
pub fn get_default_interpreter() -> String {
    let store = get_store_guard();
    store.default_interpreter.clone()
}

pub fn set_default_interpreter(def_interpreter: String) {
    let mut store = get_store_guard();
    store.default_interpreter = def_interpreter;
    write_store(&*store);
}

pub fn get_notif_timeout() -> i32 {
    let store = get_store_guard();
    store.notif_timeout
}

pub fn set_notif_timeout(timeout: i32) {
    let mut store = get_store_guard();
    store.notif_timeout = timeout;
    write_store(&*store);
}

pub fn get_timeout_enabled() -> bool {
    let store = get_store_guard();
    store.timeout_enabled
}

pub fn set_timeout_enabled(enabled: bool) {
    let mut store = get_store_guard();
    store.timeout_enabled = enabled;
    write_store(&*store);
}

pub fn get_scale_factor() -> f32 {
    let store = get_store_guard();
    store.scale_factor
}

pub fn set_scale_factor(sf: f32) {
    let mut store = get_store_guard();
    store.scale_factor = sf;
    write_store(&*store);
}

pub fn get_language() -> String {
    let store = get_store_guard();
    store.language.clone()
}

pub fn set_language(lang: String) {
    let mut store = get_store_guard();
    store.language = lang;
    write_store(&*store);
}

pub fn get_scripts() -> Vec<Script> {
    // We return a cloned vec as it should never hold more than a
    // few dozens values (knowing the use of the software).
    let scripts = get_store_guard().scripts.clone();
    scripts.iter().map(|ss| ss.into()).collect()
}

pub fn add_script(script: Script) -> String {
    let mut store = get_store_guard();
    let mut stored_script: StoredScript = script.into();
    stored_script.id = Uuid::new_v4().to_string();
    store.scripts.push(stored_script.clone());
    write_store(&*store);
    stored_script.id
}

pub fn update_script(script_index: usize, script: Script) {
    let mut store = get_store_guard();
    // Using the index from the VecModel as it should reflect the stored vec.
    store.scripts[script_index] = script.into();
    write_store(&*store);
}

pub fn remove_script(script_index: i32) {
    let mut store = get_store_guard();
    store.scripts.remove(script_index as usize);
    write_store(&*store);
}
