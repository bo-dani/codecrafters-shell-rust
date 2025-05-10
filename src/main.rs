use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while1};
use nom::character::complete::space1;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{IResult, Parser};
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

#[derive(Debug)]
enum Token {
    Arg(String),
    DoubleQuotedArg(String),
    Whitespace,
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

fn process_tokens(tokens: Vec<Token>) -> Vec<String> {
    let mut p: Vec<String> = Vec::new();
    let mut sb: String = String::new();
    for token in tokens {
        match token {
            Token::Arg(t) | Token::DoubleQuotedArg(t) => {
                sb.push_str(&t);
            }
            Token::Whitespace => {
                if !sb.is_empty() {
                    p.push(sb.clone());
                    sb.clear();
                }
            }
        }
    }
    if !sb.is_empty() {
        p.push(sb);
    }
    return p;
}

/// Handle the shell-builtin `type` command.
fn handle_type_cmd(args: Vec<String>) {
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
fn handle_echo_cmd(args: Vec<String>) {
    println!("{}", args.join(" "));
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

fn parse_unquoted_arg(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(input)
}

fn parse_single_quoted_arg(input: &str) -> IResult<&str, &str> {
    delimited(tag("'"), is_not("'"), tag("'")).parse(input)
}

fn parse_double_quoted_arg(input: &str) -> IResult<&str, &str> {
    delimited(tag("\""), is_not("\""), tag("\"")).parse(input)
}

fn parse_args(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt((
        map(space1, |_| Token::Whitespace),
        alt((
            map(parse_single_quoted_arg, |s: &str| {
                Token::Arg(String::from_str(s).unwrap())
            }),
            map(parse_double_quoted_arg, |s: &str| {
                Token::DoubleQuotedArg(String::from_str(s).unwrap())
            }),
            map(parse_unquoted_arg, |s: &str| {
                Token::Arg(String::from_str(s).unwrap())
            }),
        )),
    )))
    .parse(input)
}

fn parse_command(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(input)
}

fn parse_input(input: &str) -> IResult<&str, (&str, Vec<Token>)> {
    let (input, cmd) = parse_command(input)?;
    let (input, args) = parse_args(input)?;
    Ok((input, (cmd, args)))
}

fn main() -> ExitCode {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let (_, (cmd, args)) = parse_input(&input).unwrap();
        let args = process_tokens(args);
        match cmd {
            "exit" => {
                return handle_exit_cmd(args);
            }
            "type" => {
                handle_type_cmd(args);
            }
            "echo" => {
                handle_echo_cmd(args);
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
