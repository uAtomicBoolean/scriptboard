// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use app::store;
use dotenv::dotenv;
use log::error;
use slint::VecModel;
use std::{error::Error, rc::Rc, time::SystemTime};

mod app;
mod ui;

slint::include_modules!();

fn setup_logger() -> Result<(), fern::InitError> {
    let dispatch = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!(
            "[{}] {} {} {}",
            humantime::format_rfc3339_seconds(SystemTime::now()),
            record.level(),
            record.target(),
            message,
        ))
    });

    if std::env::var("ENV").unwrap_or("dev".to_string()) == "dev" {
        dispatch
            .level(log::LevelFilter::Debug)
            .chain(std::io::stdout())
            .apply()?;
        return Ok(());
    }

    let mut log_file_path = match dirs::data_local_dir() {
        Some(p) => p,
        None => {
            dispatch.level(log::LevelFilter::Info).apply()?;
            return Ok(());
        }
    };

    log_file_path.push("scriptboard.log");
    dispatch
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_file_path)?)
        .apply()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Checking for a dotenv file only in dev environment.
    if std::env::var("ENV").unwrap_or("dev".to_string()) == "dev" {
        dotenv().ok();
    }
    setup_logger()?;

    let stored_scripts = app::store::get_scripts();
    let scripts = Rc::new(VecModel::<Script>::from(stored_scripts));

    let ui = Scriptboard::new()?;

    ui::components::add_script_modal::init_ui(ui.as_weak(), scripts.clone());
    ui::components::notification_overlay::init_ui(ui.as_weak());
    ui::components::scripts_settings_modal::init_ui(ui.as_weak(), scripts.clone());
    ui::pages::about_page::init_ui(ui.as_weak());
    ui::pages::main_page::init_ui(ui.as_weak(), scripts.clone());
    ui::pages::settings_page::init_ui(ui.as_weak());

    let app_theme = ui.global::<UAppTheme>();
    app_theme.set_scale_factor(store::get_scale_factor());

    if let Err(err) = slint::select_bundled_translation(&store::get_language()) {
        error!("Couldn't select the bundled translation.");
        error!("{}", err.to_string());
        std::process::exit(1);
    }

    ui.run()?;

    Ok(())
}
