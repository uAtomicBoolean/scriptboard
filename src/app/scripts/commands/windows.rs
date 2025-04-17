use super::ScriptOutput;
use crate::Script;
use shlex;
use std::os::windows::process::CommandExt;
use std::process::Command;

pub fn execute(script: &Script) -> ScriptOutput {
    let output = get_command(&script).output();

    match output {
        Ok(out) => {
            return ScriptOutput {
                logs: String::from_utf8_lossy(&out.stdout).to_string(),
                status_code: out.status.code().unwrap_or_default(),
            };
        }
        Err(err) => {
            return ScriptOutput {
                logs: err.to_string(),
                status_code: 1,
            };
        }
    }
}

fn get_command(script: &Script) -> Command {
    let parsed_args = match shlex::split(&script.args) {
        Some(pargs) => pargs,
        None => vec![String::new()],
    };

    // Ask for admin password
    // Start-Process cmd -ArgumentList "/C your_command_here" -Verb RunAs
    // Example with custom interpreter :
    // Start-Process cmd -ArgumentList "/C python -c \"print('Hello, World!')\"" -Verb RunAs

    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg("-File")
            .arg(&script.path)
            .args(parsed_args)
            .creation_flags(0x08000000);
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
        .creation_flags(0x08000000);

    cmd
}
