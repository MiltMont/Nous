use std::fmt::format;

use crate::{ast::Expression, lexer::Token};
use miette::Diagnostic;
use thiserror::Error as ThisError;

pub type Result<T> = core::result::Result<T, Error>;

/// Multiple error types
#[derive(Debug, ThisError, Diagnostic)]
#[error("Error")]
pub enum Error {
    /// Parser errors
    #[error("{message:?}. \n\tUnexpected token. Expected {expected:?}, but found {found:?}")]
    UnexpectedToken {
        message: Option<String>,
        expected: Token,
        found: Token,
    },

    #[error("Preedence error, the token {found:?} is not in the precedence table.")]
    Precedence { found: Token },

    #[error("Malformed factor, missing {missing:?} but found {found:?}")]
    MalformedFactor {
        missing: Option<Token>,
        found: Token,
    },

    #[error("{found:?} is not a binary operator")]
    NotBinop { found: Token },

    #[error("{found:?} is not a unary operator")]
    NotUnop { found: Token },

    /// Io errors
    #[diagnostic()]
    IoError(#[from] std::io::Error),
}
