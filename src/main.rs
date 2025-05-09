use std::{
    io::{self, Write},
    process::ExitCode,
};

use regex::Regex;

static BUILTIN_CMDS: &'static [&'static str] = &["echo", "exit", "type"];

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
            if BUILTIN_CMDS.contains(&caps[1].trim()) {
                println!("{} is a shell builtin", caps[1].trim());
            } else {
                println!("{}: not found", caps[1].trim());
            }
        } else {
            println!("{}: command not found", input.trim());
        }
    }
}
