use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

const COMMANDS: &[(&str, &str)] = &[
    ("get", "get <key>"),
    ("set", "set <key> <value>"),
    ("del", "del <key>"),
    ("exists", "exists <key>"),
    ("total", "total"),
];

pub struct ShellHelper;

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let word = &line[..pos];
        let completions = COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(word))
            .map(|(cmd, usage)| Pair {
                display: usage.to_string(),
                replacement: cmd.to_string(),
            })
            .collect();
        Ok((0, completions))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context) -> Option<String> {
        if line.trim().is_empty() {
            return None;
        }
        let word = line.trim();
        COMMANDS
            .iter()
            .find(|(cmd, _)| cmd.starts_with(word))
            .map(|(_, usage)| usage[word.len()..].to_string())
    }
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}
impl Helper for ShellHelper {}
