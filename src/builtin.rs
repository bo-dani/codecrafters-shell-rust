use std::process::ExitCode;

/// Handle the shell-builtin `type` command.
pub fn handle_type_cmd(args: Vec<String>) {
    if args.len() != 1 {
        // TODO Print out error to user.
        return;
    }

    let arg = &args[0];
    match CommandType::from_str(&arg) {
        Ok(CommandType::Builtin) => println!("{} is a shell builtin", &arg),
        Ok(CommandType::Executable(path)) => println!("{} is {}", &arg, path.to_str().unwrap()),
        Err(_) => println!("{}: not found", &arg),
    }
}

/// Handle the shell-builtin `echo` command.
pub fn handle_echo_cmd(args: Vec<String>) {
    println!("{}", args.join(" "));
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
