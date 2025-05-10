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

fn process_tokens(tokens: Vec<Token>) -> Vec<String> {
    let mut p: Vec<String> = Vec::new();
    let mut sb: String = String::new();
    for token in tokens {
        match token {
            Token::DoubleQuotedArg(t) => {
                sb.push_str(&t);
            }
            Token::Arg(t) => {
                let escaped = t.replace("\\", "");
                sb.push_str(&escaped);
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
    println!("{:?}", args);
    Ok((input, (cmd, process_tokens(args))))
}
