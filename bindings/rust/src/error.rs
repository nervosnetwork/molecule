use std::{error, fmt, result};

use crate::ItemId;

#[derive(Debug)]
pub enum VerificationError {
    TotalSizeNotMatch(String, usize, usize),
    TotalSizeNotAsExpected(String, usize, usize, usize),
    HeaderIsBroken(String, usize, usize),
    UnknownItem(String, usize, ItemId),
    FirstOffsetIsBroken(String, usize),
    FirstOffsetIsShort(String, usize, usize),
    FirstOffsetIsOverflow(String, usize, usize),
    DataIsShort(String, usize, usize),
    OffsetsNotMatch(String),
    FieldIsBroken(String, usize),
}

pub type VerificationResult<T> = result::Result<T, VerificationError>;

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::TotalSizeNotMatch(st, expected, actual) => {
                write!(
                    f,
                    "{} total size doesn't match, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
            VerificationError::TotalSizeNotAsExpected(st, min, max, actual) => {
                write!(
                    f,
                    "{} total size doesn't match, expect [{}..{}], actual {}",
                    st, min, max, actual
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
            VerificationError::FirstOffsetIsBroken(st, actual) => {
                write!(f, "{} an offset is broken, actual {}", st, actual)?;
            }
            VerificationError::FirstOffsetIsShort(st, expected, actual) => {
                write!(
                    f,
                    "{} first offset is short, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
            VerificationError::FirstOffsetIsOverflow(st, expected, actual) => {
                write!(
                    f,
                    "{} first offset is overflow, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
            VerificationError::DataIsShort(st, expected, actual) => {
                write!(
                    f,
                    "{} data is short, expect {}, actual {}",
                    st, expected, actual
                )?;
            }
            VerificationError::OffsetsNotMatch(st) => {
                write!(f, "{} some offsets is not match", st)?;
            }
            VerificationError::FieldIsBroken(st, actual) => {
                write!(f, "{} field#{} is broken", st, actual)?;
            }
        }
        Ok(())
    }
}

impl error::Error for VerificationError {}

#[derive(Debug)]
pub enum Error {
    Verification(VerificationError),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Verification(err) => {
                write!(f, "VerificationError: {}", err)?;
            }
        }
        Ok(())
    }
}

impl error::Error for Error {}
