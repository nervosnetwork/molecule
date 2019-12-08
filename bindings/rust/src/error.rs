use core::{fmt, result};

use crate::Number;

#[doc(hidden)]
#[macro_export]
macro_rules! verification_error {
    ($self:ident, $err:ident $(, $args:expr )*) => {
        Err($crate::error::VerificationError::$err($self::NAME $(, $args )*))
    }
}

#[derive(Debug, Fail)]
pub enum VerificationError {
    TotalSizeNotMatch(&'static str, usize, usize),
    HeaderIsBroken(&'static str, usize, usize),
    UnknownItem(&'static str, usize, Number),
    OffsetsNotMatch(&'static str),
    FieldCountNotMatch(&'static str, usize, usize),
}

pub type VerificationResult<T> = result::Result<T, VerificationError>;

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VerificationError::TotalSizeNotMatch(st, expected, actual) => {
                write!(
                    f,
                    "{} total size doesn't match, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
            VerificationError::HeaderIsBroken(st, expected, actual) => {
                write!(
                    f,
                    "{} total size is not enough for header, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
            VerificationError::UnknownItem(st, size, actual) => {
                write!(
                    f,
                    "{} item id (={}) is an unknown id, only has {} kind of items",
                    st, actual, size
                )?;
            }
            VerificationError::OffsetsNotMatch(st) => {
                write!(f, "{} some offsets is not match", st)?;
            }
            VerificationError::FieldCountNotMatch(st, expected, actual) => {
                write!(
                    f,
                    "{} field count doesn't match, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    Verification(VerificationError),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Verification(err) => {
                write!(f, "VerificationError: {}", err)?;
            }
        }
        Ok(())
    }
}
