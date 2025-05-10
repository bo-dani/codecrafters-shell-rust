use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
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
        } else if let Ok(Some(executable)) = is_executable(cmd) {
            Ok(CommandType::Executable(executable))
        } else {
            Err(())
        }
    }
}

fn is_executable(cmd: &str) -> Result<Option<PathBuf>> {
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

fn handle_type_cmd(param: Option<&str>) {
    if param.is_none() {
        return;
    }

    let param = param.unwrap();

    match CommandType::from_str(param) {
        Ok(CommandType::Builtin) => println!("{} is a shell builtin", param),
        Ok(CommandType::Executable(path)) => println!("{} is {}", param, path.to_str().unwrap()),
        Err(_) => println!("{}: not found", param),
    }
}

fn handle_echo_cmd(param: Option<(&str, &str)>) {
    if let Some((_, s)) = param {
        println!("{}", s);
    } else {
        println!("");
    }
}

fn handle_exit_cmd(param: Option<&str>) -> ExitCode {
    if param.is_none() {
        ExitCode::SUCCESS
    } else if let Ok(retval) = param.unwrap().parse::<u8>() {
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
        if input.starts_with("exit ") || input.trim() == "exit" {
            return handle_exit_cmd(input.split_ascii_whitespace().nth(1));
        } else if input.starts_with("type ") || input.trim() == "type" {
            handle_type_cmd(input.split_ascii_whitespace().nth(1));
        } else if input.starts_with("echo ") || input.trim() == "echo" {
            handle_echo_cmd(input.split_once(" "));
        }
    }
}
