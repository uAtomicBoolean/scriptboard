use crate::app::store::StoredScript;
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "linux")]
pub fn get_linux_command(script: &StoredScript, parsed_args: Vec<String>) -> Command {
    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path)
            .args(parsed_args)
            .stdout(Stdio::piped());
        return cmd;
    }

    let mut cmd = Command::new("pkexec");
    cmd.args([&script.interpreter, &script.path])
        .args(parsed_args)
        .stdout(Stdio::piped());

    cmd
}

#[cfg(target_os = "macos")]
pub fn get_macos_command(script: &StoredScript, parsed_args: Vec<String>) -> Command {
    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path)
            .args(parsed_args)
            .stdout(Stdio::piped());
        return cmd;
    }

    let osascript_arg = format!(
        "do shell script \"{} {} {}\" with prompt \"{}\" with administrator privileges",
        &script.interpreter,
        &script.path,
        parsed_args.join(" "),
        &script.name,
    );

    let mut cmd = Command::new("osascript");
    cmd.arg("-e").arg(osascript_arg).stdout(Stdio::piped());

    cmd
}

#[cfg(target_os = "windows")]
pub fn get_windows_command(script: &StoredScript, parsed_args: Vec<String>) -> Command {
    // Ask for admin password
    // Start-Process cmd -ArgumentList "/C your_command_here" -Verb RunAs
    // Example with custom interpreter :
    // Start-Process cmd -ArgumentList "/C python -c \"print('Hello, World!')\"" -Verb RunAs

    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg("-File")
            .arg(&script.path)
            .args(parsed_args)
            .creation_flags(0x08000000)
            .stdout(Stdio::piped());
        return cmd;
    }

    let script_args = format!(
        "\"/C {} {} {}\"",
        &script.interpreter,
        &script.path,
        parsed_args.join(" ")
    );

    let mut cmd: Command = Command::new("Start-Process");
    cmd.arg("cmd")
        .arg("-ArgumentList")
        .arg(script_args)
        .creation_flags(0x08000000)
        .stdout(Stdio::piped());

    cmd
}
