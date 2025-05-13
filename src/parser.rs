use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_while1};
use nom::character::complete::char;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many0_count;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::{IResult, Parser};
use std::str::FromStr;

#[derive(Debug)]
enum Token {
    Arg(String),
    SingleQuotedArg(String),
    DoubleQuotedArg(String),
    Whitespace,
    EscapedCharacter(char),
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

fn process_tokens(tokens: Vec<Token>) -> Vec<String> {
    let mut p: Vec<String> = Vec::new();
    let mut sb: String = String::new();
    let mut first_whitespace = true;
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
                if !first_whitespace {
                    p.push(sb.clone());
                    sb.clear();
                }
                first_whitespace = false;
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

fn parse_redirection(input: &str) -> IResult<&str, &str> {
    tag(">")(input)
}

fn parse_args(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt((
        map(space1, |_| Token::Whitespace),
        map(parse_single_quoted_input, |s: &str| {
            Token::SingleQuotedArg(String::from_str(s).expect("The string cannot be ill-formatted"))
        }),
        map(parse_double_quoted_input, |s: &str| {
            Token::DoubleQuotedArg(String::from_str(s).expect("The string cannot be ill-formatted"))
        }),
        map(parse_unquoted_input, |s: &str| {
            Token::Arg(String::from_str(s).expect("The string cannot be ill-formatted"))
        }),
        map(parse_escaped_char, |s: &str| {
            Token::EscapedCharacter(
                s.chars()
                    .into_iter()
                    .next()
                    .expect("The parser returns two characters when it successful"),
            )
        }),
    )))
    .parse(input)
}

fn parse_command(input: &str) -> IResult<&str, &str> {
    alt((
        parse_single_quoted_input,
        parse_double_quoted_input,
        parse_unquoted_input,
    ))
    .parse(input)
}

pub fn parse_input(input: &str) -> IResult<&str, (&str, Vec<String>)> {
    let (input, cmd) = parse_command(input)?;
    let (input, args) = parse_args(input)?;
    println!("{:?}", args);
    Ok((input, (cmd, process_tokens(args))))
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
