use anyhow::{Context, Result};
use regex::Regex;
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
    Unknown,
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
            Ok(CommandType::Unknown)
        }
    }
}

fn is_executable(cmd: &str) -> Result<Option<PathBuf>> {
    for path in split_path() {
        for f in fs::read_dir(&path).context("Directory in PATH cannot be read")? {
            if f?
                .path()
                .to_str()
                .expect("File should be a UTF-8 valid string")
                .ends_with(cmd)
            {
                return Ok(Some(path));
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

fn handle_type_cmd(param: &str) {
    if BUILTIN_CMDS.contains(&param) {
        println!("{} is a shell builtin", param);
    } else if let Ok(Some(executable)) = is_executable(param) {
        println!("{} is {}", param, executable.to_str().unwrap());
    } else {
        println!("{}: not found", param);
    }
}

fn main() -> ExitCode {
    let exit_re: Regex = Regex::new(r"exit ([0-9]+)").unwrap();
    let echo_re: Regex = Regex::new(r"echo (.+)").unwrap();
    let type_re: Regex = Regex::new(r"type (.+)").unwrap();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if let Some(caps) = exit_re.captures(&input) {
            return ExitCode::from(
                caps[1]
                    .parse::<u8>()
                    .expect("The regex already makes sure that this is a valid usize"),
            );
        } else if let Some(caps) = echo_re.captures(&input) {
            println!("{}", caps[1].trim());
        } else if let Some(caps) = type_re.captures(&input) {
            handle_type_cmd(caps[1].trim());
        } else {
            println!("{}: command not found", input.trim());
        }
    }
}
