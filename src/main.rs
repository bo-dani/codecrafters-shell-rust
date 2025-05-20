use anyhow::Result;
use parser::Redirection;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write as IoWrite};
use std::path::Path;
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
fn handle_type_cmd(args: &[String], redirection: Redirection) {
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

    match redirection {
        Redirection::None | Redirection::Stderr(_) => {
            print!("{}", output);
        }
        Redirection::Stdout(filename) => {
            if let Ok(mut file) = File::create(filename) {
                file.write(output.as_bytes()).unwrap();
                file.flush().unwrap();
            }
        }
    }
}

/// Handle the shell-builtin `echo` command.
fn handle_echo_cmd(args: &[String], redirection: Redirection) {
    let binding = format!("{}\n", args.join(" "));
    let echo = binding.as_str();
    match redirection {
        Redirection::None => {
            print!("{}", echo);
        }
        Redirection::Stderr(filename) => {
            let path = Path::new(&filename);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                match File::create(&filename) {
                    Ok(_) => print!("{}", echo),
                    Err(e) => println!("error creating file: {}", e),
                }
            }
        }
        Redirection::Stdout(filename) => match File::create(filename) {
            Ok(mut file) => {
                file.write(echo.as_bytes()).unwrap();
                file.flush().unwrap();
            }
            Err(e) => println!("error creating file: {}", e),
        },
    }
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

fn handle_executable_cmd(cmd: &str, args: Vec<String>, redirection: Redirection) {
    let mut binding = Command::new(cmd);
    let mut command = binding.args(args);
    match redirection {
        Redirection::None => {}
        Redirection::Stdout(filename) => {
            if let Ok(file) = File::create(filename) {
                command = command.stdout(file);
            }
        }
        Redirection::Stderr(filename) => {
            if let Ok(file) = File::create(filename) {
                command = command.stderr(file);
            }
        }
    }
    command.status().expect("failed to exceute process");
}

fn main() -> ExitCode {
    loop {
        print!("$ ");
        std::io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if let Ok((cmd, args, redirection)) = parser::parse_input(&input) {
            match cmd {
                "exit" => {
                    return handle_exit_cmd(args);
                }
                "type" => {
                    handle_type_cmd(args.as_slice(), redirection);
                }
                "echo" => {
                    handle_echo_cmd(args.as_slice(), redirection);
                }
                "" => {
                    continue;
                }
                _ => {
                    if let Ok(Some(_)) = get_executable_path(cmd) {
                        handle_executable_cmd(cmd, args, redirection);
                    } else {
                        println!("{}: command not found", cmd);
                    }
                }
            }
        }
    }
}
