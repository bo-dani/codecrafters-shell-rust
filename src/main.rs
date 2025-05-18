use anyhow::Result;
use std::env;
use std::fmt::Write;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::str::FromStr;

mod parser;

static BUILTIN_CMDS: &'static [&'static str] = &["echo", "exit", "type"];

enum CommandType {
    Builtin,
    Executable(PathBuf),
}

impl FromStr for CommandType {
    type Err = ();

    fn from_str(cmd: &str) -> Result<CommandType, Self::Err> {
        if BUILTIN_CMDS.contains(&cmd) {
            Ok(CommandType::Builtin)
        } else if let Ok(Some(executable)) = get_executable_path(cmd) {
            Ok(CommandType::Executable(executable))
        } else {
            Err(())
        }
    }
}

fn get_executable_path(cmd: &str) -> Result<Option<PathBuf>> {
    for path in split_path() {
        if !fs::exists(&path).unwrap() {
            continue;
        }
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if cmd == entry.file_name().to_str().unwrap() {
                return Ok(Some(entry.path()));
            }
        }
    }
    Ok(None)
}

fn split_path() -> Vec<PathBuf> {
    let path: String = env::var("PATH").expect("PATH environment variable should always exist");
    let mut paths: Vec<PathBuf> = Vec::new();
    for p in path.split(":") {
        paths.push(PathBuf::from(&p));
    }
    paths
}

/// Handle the shell-builtin `type` command.
fn handle_type_cmd(args: &[String], file: &mut impl Write) {
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
    file.write_str(output.as_str()).unwrap();
}

/// Handle the shell-builtin `echo` command.
fn handle_echo_cmd(args: &[String], file: &mut impl Write) {
    file.write_str(format!("{}\n", args.join(" ")).as_str())
        .unwrap();
}

/// Handle the shell-builtin `exit` command.
fn handle_exit_cmd(args: Vec<String>) -> ExitCode {
    if args.is_empty() {
        ExitCode::SUCCESS
    } else if let Ok(retval) = args[0].parse::<u8>() {
        ExitCode::from(retval)
    } else {
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Ok((cmd, args, mut redirection)) = parser::parse_input(&input) {
            match cmd {
                "exit" => {
                    return handle_exit_cmd(args);
                }
                "type" => {
                    handle_type_cmd(args.as_slice(), &mut redirection);
                }
                "echo" => {
                    handle_echo_cmd(args.as_slice(), &mut redirection);
                }
                "" => {
                    continue;
                }
                _ => {
                    if let Ok(Some(_)) = get_executable_path(cmd) {
                        Command::new(cmd)
                            .args(args)
                            .status()
                            .expect("failed to execute process");
                    } else {
                        println!("{}: command not found", cmd);
                    }
                }
            }
        }
    }
}
