#[cfg(target_os = "windows")]
fn build_rc_file() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("res/icon.ico");
    res.compile()
        .expect("Error while compiling rc file informations.");
}

fn main() {
    #[cfg(target_os = "windows")]
    build_rc_file();

    slint_build::compile("ui/scriptboard.slint").expect("Slint build failed");
}
