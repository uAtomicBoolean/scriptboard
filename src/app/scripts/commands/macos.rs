use super::ScriptOutput;
use crate::Script;
use shlex;
use std::process::Command;

pub fn execute(script: &Script) -> ScriptOutput {
    let cmd = get_command(&script).output();

    match cmd {
        Ok(output) => {
            return ScriptOutput {
                logs: String::from_utf8_lossy(&output.stdout).to_string(),
                status_code: output.status.code().unwrap_or_default(),
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

    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path).args(parsed_args);
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
    cmd.arg("-e").arg(osascript_arg);

    cmd
}
