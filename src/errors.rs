use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HpError {
    NumberOfValues(String, usize, usize),
    OutOfContext(String, String),
}

impl Display for HpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NumberOfValues(arg, got, expected) => write!(f, "ERROR: In argument `{arg}`, expected {expected} value/s, received {got}."),
            Self::OutOfContext(arg, parent) => write!(f, "ERROR: Out of context arugment, because '{arg}' is a subcommand of '{parent}' and '{parent}' is not present in the command."),
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
