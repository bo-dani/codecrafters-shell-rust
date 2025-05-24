use crate::autocomplete::Autocompleter;
use crate::builtin::BUILTIN_CMDS;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use std::io::Write;
use std::process::ExitCode;

mod autocomplete;
mod builtin;
mod command;
mod fs;
mod parser;

static PROMPT: &'static str = "$ ";

fn main() -> ExitCode {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(rustyline::CompletionType::List)
        .edit_mode(rustyline::EditMode::Vi)
        .build();

    let mut cmds = Vec::new();
    cmds.extend(command::get_executables());
    for builtin in BUILTIN_CMDS {
        cmds.push(builtin.to_string());
    }

    let autocomplete = Autocompleter::new(cmds.as_slice());
    let mut editor: Editor<Autocompleter, _> = Editor::with_config(config).unwrap();
    editor.set_helper(Some(autocomplete));

    loop {
        std::io::stdout().flush().unwrap();

        // Wait for user input
        let input = match editor.readline(PROMPT) {
            Ok(line) => line,
            Err(e) => match e {
                ReadlineError::Interrupted => {
                    return ExitCode::FAILURE;
                }
                _ => continue,
            },
        };

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
