use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while1};
use nom::character::complete::space1;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{IResult, Parser};
use std::str::FromStr;

#[derive(Debug)]
enum Token {
    Arg(String),
    DoubleQuotedArg(String),
    Whitespace,
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
            if idx == arg.len() - 2 {
                escaped.push(next);
            }
            continue;
        }
        should_escape = false;
        escaped.push(cur);
    }
    escaped
}

fn process_tokens(tokens: Vec<Token>) -> Vec<String> {
    let mut p: Vec<String> = Vec::new();
    let mut sb: String = String::new();
    let mut first_whitespace = true;
    for token in tokens {
        match token {
            Token::DoubleQuotedArg(t) => {
                sb.push_str(&t);
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
        }
    }
    if !sb.is_empty() {
        p.push(sb);
    }
    return p;
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

pub fn parse_input(input: &str) -> IResult<&str, (&str, Vec<String>)> {
    let (input, cmd) = parse_command(input)?;
    let (input, args) = parse_args(input)?;
    Ok((input, (cmd, process_tokens(args))))
}
