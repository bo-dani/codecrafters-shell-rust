use rustyline::completion::{extract_word, Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::Context;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::collections::HashSet;

const DEFAULT_BREAK_CHARS: [char; 3] = [' ', '\t', '\n'];

#[derive(Hash, Debug, PartialEq, Eq)]
struct Command {
    cmd: String,
    pre_cmd: String,
}

impl Command {
    pub fn new(cmd: &str, pre_cmd: &str) -> Self {
        Self {
            cmd: cmd.into(),
            pre_cmd: pre_cmd.into(),
        }
    }
}

#[derive(Helper, Hinter, Validator, Highlighter)]
pub struct Autocompleter {
    cmds: HashSet<Command>,
}

impl Autocompleter {
    pub fn new(cmds: &[String]) -> Self {
        let mut map = HashSet::new();
        for cmd in cmds {
            map.insert(Command::new(cmd, ""));
        }

        Self { cmds: map }
    }

    pub fn find_matches(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, word) = extract_word(line, pos, None, |c| DEFAULT_BREAK_CHARS.contains(&c));
        let pre_cmd = line[..start].trim();

        let matches = self
            .cmds
            .iter()
            .filter_map(|hint| {
                if hint.cmd.starts_with(word) && pre_cmd == &hint.pre_cmd {
                    let mut replacement = hint.cmd.clone();
                    replacement += " ";
                    Some(Pair {
                        display: hint.cmd.to_string(),
                        replacement: replacement.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok((start, matches))
    }
}

impl Completer for Autocompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match self.find_matches(line, pos) {
            Ok((start, matches)) => Ok((start, matches)),
            Err(e) => Err(e),
        }
    }
}
