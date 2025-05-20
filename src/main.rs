use std::io::{self, Write as IoWrite};
use std::process::ExitCode;

mod builtin;
mod command;
mod fs;
mod parser;

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
                    return builtin::handle_exit_cmd(args);
                }
                "type" => {
                    builtin::handle_type_cmd(args.as_slice(), redirection);
                }
                "echo" => {
                    builtin::handle_echo_cmd(args.as_slice(), redirection);
                }
                "" => {
                    continue;
                }
                _ => {
                    if let Ok(Some(_)) = fs::get_executable_path(cmd) {
                        command::handle_executable_cmd(cmd, args, redirection);
                    } else {
                        println!("{}: command not found", cmd);
                    }
                }
            }
        }
    }
}
