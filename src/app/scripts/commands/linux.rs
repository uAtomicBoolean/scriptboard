use super::ScriptOutput;
use crate::Script;
use shlex;
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

    if !script.need_admin {
        let mut cmd = Command::new(&script.interpreter);
        cmd.arg(&script.path).args(parsed_args);
        return cmd;
    }

    let mut cmd = Command::new("pkexec");
    cmd.args([&script.interpreter, &script.path])
        .args(parsed_args);

    cmd
}
