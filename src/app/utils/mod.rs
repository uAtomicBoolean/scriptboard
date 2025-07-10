use log::error;
use std::path::PathBuf;

fn check_create_local_data_dir() -> PathBuf {
    let mut local_data_dir = match dirs::data_local_dir() {
        Some(p) => p,
        None => {
            error!("Couldn't find the local data dir.");
            std::process::exit(1);
        }
    };

    local_data_dir.push(PathBuf::from("scriptboard"));

    if !local_data_dir.exists() {
        match std::fs::create_dir(&local_data_dir) {
            Ok(_) => (),
            Err(err) => {
                error!("Coulnd't create the local data directory for scriptboard.");
                error!("{}", err.to_string());
                std::process::exit(1);
            }
        };
    }
    local_data_dir
}

pub fn get_store_path() -> PathBuf {
    let mut store_path = check_create_local_data_dir();
    store_path.push("store.json");
    store_path
}

pub fn get_log_file_path() -> PathBuf {
    let mut log_file_path = check_create_local_data_dir();
    log_file_path.push("scriptboard.log");
    log_file_path
}
