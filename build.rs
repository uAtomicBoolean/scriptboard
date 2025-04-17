use dotenv::dotenv;

#[cfg(target_os = "windows")]
fn build_rc_file() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("res/icon.ico");
    res.compile()
        .expect("Error while compiling rc file informations.");
}

fn main() {
    if std::env::var("ENV").unwrap_or("dev".to_string()) == "dev" {
        match dotenv() {
            Ok(path) => println!("{}", path.to_string_lossy()),
            Err(err) => {
                println!("Couldn't find the .env file.");
                println!("{}", err.to_string());
                panic!();
            }
        }
    }

    #[cfg(target_os = "windows")]
    build_rc_file();

    let sleek_ui_path = match std::env::var("SLEEK_UI_PATH") {
        Ok(sup) => std::path::PathBuf::from(sup),
        Err(err) => {
            println!("Couldn't get SLEEK_UI_PATH env var.");
            println!("{}", err.to_string());
            panic!();
        }
    };

    let library_paths = std::collections::HashMap::from([("sleek-ui".to_string(), sleek_ui_path)]);
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(library_paths)
        .with_bundled_translations("lang");

    slint_build::compile_with_config("ui/scriptboard.slint", config).expect("Slint build failed");
}
