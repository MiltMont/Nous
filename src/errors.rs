use crate::lexer::Token;

pub type Result<T> = core::result::Result<T, Error>;

/// Multiple error types
#[derive(Debug)]
pub enum Error {
    /// Parser errors
    UnexpectedToken {
        message: String,
        expected: Token,
        found: Token,
    },
    Precedence {
        found: Token,
    },

    MalformedFactor {
        missing: Option<Token>,
        found: Token,
    },

    NotBinop {
        found: Token,
    },

    NotUnop {
        found: Token,
    },
}
