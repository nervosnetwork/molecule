//! Explicit logical integers over existing, directly declared byte arrays.

#[cfg(feature = "schema-extensions")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "schema-extensions",
    derive(Deserialize, Serialize),
    serde(rename_all = "lowercase")
)]
pub enum ByteOrder {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "schema-extensions",
    derive(Deserialize, Serialize),
    serde(deny_unknown_fields)
)]
pub struct IntegerSpec {
    pub bits: u16,
    pub signed: bool,
    pub byte_order: ByteOrder,
}

impl IntegerSpec {
    // Width and signedness are shared by validation and plugin capabilities.
    pub(crate) const SUPPORTED_KINDS: [(u16, bool); 11] = [
        (8, false),
        (8, true),
        (16, false),
        (16, true),
        (32, false),
        (32, true),
        (64, false),
        (64, true),
        (128, false),
        (128, true),
        (256, false),
    ];

    pub fn validate(self) -> Result<(), String> {
        if Self::SUPPORTED_KINDS.contains(&(self.bits, self.signed)) {
            Ok(())
        } else {
            Err(format!(
                "E_INT_WIDTH: unsupported {}{}",
                if self.signed { "i" } else { "u" },
                self.bits
            ))
        }
    }

    pub fn byte_width(self) -> usize {
        usize::from(self.bits) / 8
    }
}

impl std::str::FromStr for IntegerSpec {
    type Err = String;

    fn from_str(literal: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!("E_INT_TYPE: invalid integer type {literal:?}; expected e.g. u32le or i64be")
        };
        let (number, byte_order) = if let Some(number) = literal.strip_suffix("le") {
            (number, ByteOrder::Little)
        } else if let Some(number) = literal.strip_suffix("be") {
            (number, ByteOrder::Big)
        } else if matches!(literal, "u8" | "i8") {
            (literal, ByteOrder::Little)
        } else {
            return Err(invalid());
        };
        let (signed, width) = match number.as_bytes().first() {
            Some(b'u') => (false, &number[1..]),
            Some(b'i') => (true, &number[1..]),
            _ => return Err(invalid()),
        };
        if width.starts_with('0') || !width.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid());
        }
        let spec = Self {
            bits: width.parse().map_err(|_| invalid())?,
            signed,
            byte_order,
        };
        spec.validate()?;
        Ok(spec)
    }
}

#[cfg(feature = "schema-extensions")]
mod metadata;
#[cfg(feature = "schema-extensions")]
pub(crate) use metadata::bind;
#[cfg(feature = "schema-extensions")]
pub use metadata::{LogicalInteger, MetadataFile};
