//! Module contaning the errors which my arise when parsing.
use std::fmt::Display;

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const NONE: &str = "\x1b[0m";

#[derive(Clone, PartialEq, Eq, Debug)]
/// Enum type containing the errors.
pub enum HpError {
    /// This error is caused by an insufficient number of values for an argument.
    NumberOfValues(String, usize, usize),
    /// This error is caused by passing a subcommand before passing its parent command.
    OutOfContext(String, String),
}

impl Display for HpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NumberOfValues(arg, got, expected) => write!(f, "{RED}ERROR{NONE}: In argument '{RED}{arg}{NONE}', expected '{GREEN}{expected}{NONE}' value/s, received '{YELLOW}{got}{NONE}'."),
            Self::OutOfContext(arg, parent) => write!(f, "{RED}ERROR{NONE}: Out of context argument, because '{YELLOW}{arg}{NONE}' is a subcommand of '{GREEN}{parent}{NONE}' and '{GREEN}{parent}{NONE}' is not present in the command."),
        }
    }
}

impl std::error::Error for HpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

#[cfg(test)]
mod tests {}
