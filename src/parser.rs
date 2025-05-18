use anyhow::bail;
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_while1};
use nom::character::complete::char;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::combinator::recognize;
use nom::multi::many0_count;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::{IResult, Parser};
use regex::Regex;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{self, Write as IoWrite};
use std::str::FromStr;

static REDIRECTION_PAT: &str = r"^(\d*)>\s*(.+)";

#[derive(Debug)]
enum Token {
    Arg(String),
    SingleQuotedArg(String),
    DoubleQuotedArg(String),
    Whitespace,
    EscapedCharacter(char),
}

#[derive(Debug)]
pub enum Redirection {
    None,
    Stdout(String),
    Stderr(String),
}

impl FmtWrite for Redirection {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self {
            Redirection::Stdout(filename) | Redirection::Stderr(filename) => {
                let mut file = File::create(filename).map_err(|_| std::fmt::Error)?;
                file.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)?;
                file.flush().map_err(|_| std::fmt::Error)?;
                Ok(())
            }
            Redirection::None => {
                let mut stdout = io::stdout();
                stdout
                    .write_all(s.as_bytes())
                    .map_err(|_| std::fmt::Error)?;
                stdout.flush().map_err(|_| std::fmt::Error)?;
                Ok(())
            }
        }
    }
}

fn escape_unquoted_arg(arg: &str) -> String {
    if !arg.contains("\\") {
        return String::from_str(arg).unwrap();
    }

    let mut escaped = String::new();
    let mut should_escape = false;
    for (idx, (cur, next)) in arg.chars().into_iter().tuple_windows().enumerate() {
        if cur == '\\' && !should_escape {
            should_escape = true;
        }

        if !should_escape {
            escaped.push(cur);
        }

        should_escape = false;
        if idx == arg.len() - 2 {
            escaped.push(next);
        }
    }
    escaped
}

fn escape_double_quoted_arg(arg: &str) -> String {
    if !arg.contains("\\") {
        return String::from_str(arg).unwrap();
    }

    arg.replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\\\n", "\n")
        .replace("\\$", "$")
}

fn process_args(tokens: Vec<Token>) -> Vec<String> {
    let mut p: Vec<String> = Vec::new();
    let mut sb: String = String::new();
    for token in tokens {
        match token {
            Token::SingleQuotedArg(t) => {
                sb.push_str(&t);
            }
            Token::DoubleQuotedArg(t) => {
                sb.push_str(&escape_double_quoted_arg(&t));
            }
            Token::Arg(t) => {
                sb.push_str(&escape_unquoted_arg(&t));
            }
            Token::Whitespace => {
                p.push(sb.clone());
                sb.clear();
            }
            Token::EscapedCharacter(c) => {
                sb.push(c);
            }
        }
    }

    if !sb.is_empty() {
        p.push(sb);
    }

    return p;
}

fn parse_unquoted_input(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace() && c != '\\')(input)
}

fn parse_single_quoted_input(input: &str) -> IResult<&str, &str> {
    delimited(tag("'"), is_not("'"), tag("'")).parse(input)
}

fn parse_double_quoted_input(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        recognize(many0_count(alt((
            is_not("\\\""),
            preceded(char('\\'), take(1usize)),
        )))),
        char('"'),
    )
    .parse(input)
}

fn parse_escaped_char(input: &str) -> IResult<&str, &str> {
    preceded(tag("\\"), take(1usize)).parse(input)
}

fn parse_redirection(input: &str) -> Redirection {
    let input = input.trim();
    if input.is_empty() {
        return Redirection::None;
    }

    let re = Regex::new(REDIRECTION_PAT).unwrap();
    if let Some(caps) = re.captures(input) {
        let fdn = if caps.get(1).is_some() {
            caps.get(1).unwrap().as_str()
        } else {
            "1"
        };

        let file = caps.get(2).unwrap().as_str();
        match fdn {
            "2" => {
                return Redirection::Stderr(file.to_string());
            }
            _ => {
                return Redirection::Stdout(file.to_string());
            }
        };
    }

    return Redirection::None;
}

fn parse_args(input: &str) -> IResult<&str, Vec<Token>> {
    let mut tokens = Vec::new();
    let mut remaining_input = input.trim();

    let re_re = Regex::new(REDIRECTION_PAT).unwrap();
    while !remaining_input.is_empty() {
        if re_re.is_match(remaining_input) {
            break;
        }

        let (input, token) = alt((
            map(space1, |_| Token::Whitespace),
            map(parse_single_quoted_input, |s: &str| {
                Token::SingleQuotedArg(s.to_string())
            }),
            map(parse_double_quoted_input, |s: &str| {
                Token::DoubleQuotedArg(s.to_string())
            }),
            map(parse_unquoted_input, |s: &str| Token::Arg(s.to_string())),
            map(parse_escaped_char, |s: &str| {
                Token::EscapedCharacter(s.chars().next().unwrap())
            }),
        ))
        .parse(remaining_input)?;

        tokens.push(token);
        remaining_input = input;
    }

    Ok((remaining_input, tokens))
}

fn parse_command(input: &str) -> IResult<&str, &str> {
    alt((
        parse_single_quoted_input,
        parse_double_quoted_input,
        parse_unquoted_input,
    ))
    .parse(input)
}

pub fn parse_input(input: &str) -> anyhow::Result<(&str, Vec<String>, Redirection)> {
    let Ok((input, cmd)) = parse_command(input) else {
        bail!("Error parsing the command");
    };

    let Ok((input, args)) = parse_args(input) else {
        bail!("Error parsing the arguments");
    };

    let redirection = parse_redirection(input);
    let args = process_args(args);
    Ok((cmd, args, redirection))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_unquoted_input() {
        assert_eq!(parse_unquoted_input("cat "), Ok(("", "cat")));
        assert_eq!(parse_unquoted_input("cat\\ "), Ok(("", "cat")));
        assert_eq!(parse_unquoted_input("cat\\ hello"), Ok(("hello", "cat")));
    }
}
