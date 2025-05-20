use super::command::CommandType;
use super::parser::Redirection;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;
use std::str::FromStr;

pub static BUILTIN_CMDS: &'static [&'static str] = &["echo", "exit", "type"];

/// Handle the shell-builtin `type` command.
pub fn handle_type_cmd(args: &[String], redirection: Redirection) {
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
            if let Ok(mut file) = std::fs::File::create(filename) {
                file.write(output.as_bytes()).unwrap();
                file.flush().unwrap();
            }
        }
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
        Redirection::Stderr(filename) => {
            let path = Path::new(&filename);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                match std::fs::File::create(&filename) {
                    Ok(_) => print!("{}", echo),
                    Err(e) => println!("error creating file: {}", e),
                }
            }
        }
        Redirection::Stdout(filename) => match std::fs::File::create(filename) {
            Ok(mut file) => {
                file.write(echo.as_bytes()).unwrap();
                file.flush().unwrap();
            }
            Err(e) => println!("error creating file: {}", e),
        },
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
