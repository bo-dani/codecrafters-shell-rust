use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{
    io::{self, Write},
    process::ExitCode,
};

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

fn handle_type_cmd(arg: Option<&str>) {
    if arg.is_none() {
        return;
    }

    let arg = arg.unwrap();

    match CommandType::from_str(arg) {
        Ok(CommandType::Builtin) => println!("{} is a shell builtin", arg),
        Ok(CommandType::Executable(path)) => println!("{} is {}", arg, path.to_str().unwrap()),
        Err(_) => println!("{}: not found", arg),
    }
}

fn handle_echo_cmd(arg: &str) {
    println!("{}", arg);
}

fn handle_exit_cmd(arg: Option<&str>) -> ExitCode {
    if arg.is_none() {
        ExitCode::SUCCESS
    } else if let Ok(retval) = arg.unwrap().parse::<u8>() {
        ExitCode::from(retval)
    } else {
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let mut parts = input.trim().split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let mut args = parts;

        match cmd {
            "exit" => {
                return handle_exit_cmd(args.nth(0));
            }
            "type" => {
                handle_type_cmd(args.nth(0));
            }
            "echo" => {
                handle_echo_cmd(args.collect::<Vec<&str>>().join(" ").trim());
            }
            "" => {
                continue;
            }
            _ => {
                if let Ok(Some(_)) = get_executable_path(cmd) {
                    Command::new(cmd)
                        .args(args.collect::<Vec<&str>>())
                        .spawn()
                        .expect("failed to execute process");
                } else {
                    println!("{}: command not found", cmd);
                }
            }
        }
    }
}
