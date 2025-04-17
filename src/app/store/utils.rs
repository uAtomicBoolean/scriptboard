use super::structs::Store;
use log::error;
use std::path::PathBuf;

pub fn write_store(store_path: &PathBuf, store: &Store) {
    let json_store = match serde_json::to_string(store) {
        Ok(jt) => jt,
        Err(err) => {
            error!("Error while converting store to json string.");
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    };

    if let Err(err) = std::fs::write(store_path, json_store) {
        error!("Couldn't write the store to its file.");
        error!("{}", err.to_string());
    }
}
