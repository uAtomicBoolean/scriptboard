use dirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::BufReader, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct StoredScript {
    pub id: i32,
    pub description: String,
    pub name: String,
    pub path: String,
}

/// Get the file storage path using dirs and the determined file name.
///
/// # Example
/// ```
/// let path = get_file_storage_path();
/// check_create_file_storage(&path);
/// ```
fn get_file_storage_path() -> PathBuf {
    let dir_option = dirs::data_local_dir();
    if let None = dir_option {
        println!("Couldn't find the local data dir.");
        std::process::exit(1);
    }
    let mut storage_path = dir_option.unwrap();
    storage_path.push(PathBuf::from("scriptboard.storage.json"));
    storage_path
}

/// Check if the file storage exists and create it if it doesn't.
///
/// # Example
/// ```
/// let path = get_file_storage_path();
/// check_create_file_storage(&path);
/// ```
fn check_create_file_storage(storage_path: &PathBuf) {
    if !storage_path.exists() {
        match fs::write(storage_path, "[]") {
            Ok(_) => (),
            Err(err) => {
                println!("Couldn't create the storage file.");
                println!("{}", err.to_string());
                std::process::exit(1);
            }
        }
    }
}

fn write_to_file_storage(storage_path: &PathBuf, scripts: Vec<StoredScript>) {
    let json_string_res = serde_json::to_string(&scripts);
    if let Err(err) = json_string_res {
        println!("Error while converting scripts to json.");
        println!("{}", err.to_string());
        std::process::exit(1);
    }

    let json_string = json_string_res.unwrap();
    match fs::write(storage_path, json_string) {
        Ok(_) => (),
        Err(err) => {
            println!("Error while writing file storage.");
            println!("{}", err.to_string());
            std::process::exit(1);
        }
    }
}

/// Read the file storage to get all scripts.
///
/// # Example
/// ```
/// let scripts = get_scripts();
/// println("{:?}", scripts);
/// ```
pub fn get_scripts() -> Vec<StoredScript> {
    let storage_path = get_file_storage_path();
    check_create_file_storage(&storage_path);

    let file_storage = fs::File::open(storage_path);
    if let Err(err) = file_storage {
        println!("Error while opening file storage.");
        println!("{}", err.to_string());
        std::process::exit(1);
    }

    let reader = BufReader::new(file_storage.unwrap());
    let storage_content: Result<Vec<StoredScript>, serde_json::Error> =
        serde_json::from_reader(reader);
    if let Err(err) = storage_content {
        println!("Error while reading file storage.");
        println!("{}", err.to_string());
        std::process::exit(1);
    }

    storage_content.unwrap()
}

/// Add a new file into the file storage.
///
/// # Example
/// ```
/// ```
pub fn store_script(new_script: StoredScript) {
    let storage_path = get_file_storage_path();
    check_create_file_storage(&storage_path);
    let mut scripts = get_scripts();
    scripts.push(new_script);

    write_to_file_storage(&storage_path, scripts);
}

/// Remove a script from the storage.
///
/// # Example
/// ```
/// ```
pub fn remove_script(script_id: i32) {
    let storage_path = get_file_storage_path();
    check_create_file_storage(&storage_path);
    let filtered_scripts: Vec<StoredScript> = get_scripts()
        .into_iter()
        .filter(|s| s.id != script_id)
        .collect();

    write_to_file_storage(&storage_path, filtered_scripts);
}
