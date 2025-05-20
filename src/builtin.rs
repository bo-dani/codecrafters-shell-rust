use crate::fs;

use super::command::CommandType;
use super::parser::Redirection;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::process::ExitCode;
use std::str::FromStr;

pub static BUILTIN_CMDS: &'static [&'static str] = &["echo", "exit", "type"];

/// Handle the shell-builtin `type` command.
pub fn handle_type_cmd(args: &[String], mut redirection: Redirection) {
    let mut output = String::new();
    for arg in args {
        if arg.trim().is_empty() {
            continue;
        }

        match CommandType::from_str(&arg) {
            Ok(CommandType::Builtin) => {
                output.push_str(format!("{} is a shell builtin\n", &arg).as_str());
            }
            Ok(CommandType::Executable(path)) => {
                output.push_str(format!("{} is {}\n", &arg, path.to_str().unwrap()).as_str());
            }
            Err(_) => {
                output.push_str(format!("{}: not found\n", &arg).as_str());
            }
        };
    }

    if let Err(e) = redirection.write_str(&output) {
        println!("{}", e);
    }
}

/// Handle the shell-builtin `echo` command.
pub fn handle_echo_cmd(args: &[String], redirection: Redirection) {
    let binding = format!("{}\n", args.join(" "));
    let echo = binding.as_str();
    match redirection {
        Redirection::None => {
            print!("{}", echo);
        }
        Redirection::Stderr(filename) | Redirection::StderrAppend(filename) => {
            fs::mkdir(&filename).unwrap();
            match fs::open(&filename, false) {
                Ok(_) => print!("{}", echo),
                Err(e) => println!("error creating file: {}", e),
            }
        }
        Redirection::Stdout(filename) => {
            fs::mkdir(&filename).unwrap();
            match fs::open(&filename, false) {
                Ok(mut file) => {
                    file.write(echo.as_bytes()).unwrap();
                }
                Err(e) => println!("error creating file: {}", e),
            }
        }
        Redirection::StdoutAppend(filename) => {
            fs::mkdir(&filename).unwrap();
            match fs::open(&filename, true) {
                Ok(mut file) => {
                    file.write(echo.as_bytes()).unwrap();
                }
                Err(e) => println!("error creating file: {}", e),
            }
        }
    }
}

/// Handle the shell-builtin `exit` command.
pub fn handle_exit_cmd(args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        ExitCode::SUCCESS
    } else if let Ok(retval) = args[0].parse::<u8>() {
        ExitCode::from(retval)
    } else {
        ExitCode::FAILURE
    }
}
