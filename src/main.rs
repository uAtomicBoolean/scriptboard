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

    let log_file_path = app::utils::get_log_file_path();
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
    let scriptboard_theme = ui.global::<ScriptboardThemes>();

    app_theme.set_scale_factor(store::get_scale_factor());
    app_theme.set_theme(scriptboard_theme.get_default_theme());

    ui.window().on_close_requested(|| {
        if let Err(err) = slint::quit_event_loop() {
            log::error!("Error when trying to quit the event loop.");
            log::error!("{}", err.to_string());
            log::error!("Forcefully killing the process.");
            std::process::exit(0);
        }
        slint::CloseRequestResponse::HideWindow
    });

    if let Err(err) = slint::select_bundled_translation(&store::get_language()) {
        error!("Couldn't select the bundled translation.");
        error!("{}", err.to_string());
        std::process::exit(1);
    }

    ui.run()?;

    Ok(())
}
