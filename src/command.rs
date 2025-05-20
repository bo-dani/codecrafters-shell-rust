use crate::builtin::BUILTIN_CMDS;
use crate::fs;
use crate::parser::Redirection;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

pub enum CommandType {
    Builtin,
    Executable(PathBuf),
}

impl FromStr for CommandType {
    type Err = ();

    fn from_str(cmd: &str) -> Result<CommandType, Self::Err> {
        if BUILTIN_CMDS.contains(&cmd) {
            Ok(CommandType::Builtin)
        } else if let Ok(Some(executable)) = fs::get_executable_path(cmd) {
            Ok(CommandType::Executable(executable))
        } else {
            Err(())
        }
    }
}

pub fn handle_executable_cmd(cmd: &str, args: Vec<String>, redirection: Redirection) {
    let mut binding = Command::new(cmd);
    let mut command = binding.args(args);
    match redirection {
        Redirection::None => {}
        Redirection::Stdout(filename) => {
            fs::mkdir(&filename).unwrap();
            if let Ok(file) = fs::open(&filename, false) {
                command = command.stdout(file);
            }
        }
        Redirection::StdoutAppend(filename) => {
            fs::mkdir(&filename).unwrap();
            if let Ok(file) = fs::open(&filename, true) {
                command = command.stdout(file);
            }
        }
        Redirection::Stderr(filename) => {
            fs::mkdir(&filename).unwrap();
            if let Ok(file) = fs::open(&filename, false) {
                command = command.stderr(file);
            }
        }
        Redirection::StderrAppend(filename) => {
            fs::mkdir(&filename).unwrap();
            if let Ok(file) = fs::open(&filename, true) {
                command = command.stderr(file);
            }
        }
    }
    command.status().expect("failed to exceute process");
}
