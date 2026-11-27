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
    pub(crate) const SUPPORTED_KINDS: [(u16, bool); 10] = [
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

    /// Normalize single-byte integers to `ByteOrder::Little` so AST, IR v2,
    /// plugin capabilities, and generated code share one canonical form.
    pub const fn canonical(self) -> Self {
        if self.bits == 8 {
            Self {
                byte_order: ByteOrder::Little,
                ..self
            }
        } else {
            self
        }
    }

    /// Single-byte integers have no multi-byte order on the wire, so 8-bit
    /// specifications compare equal regardless of `byte_order`.
    pub fn is_equivalent_to(self, other: Self) -> bool {
        self.bits == other.bits
            && self.signed == other.signed
            && (self.bits == 8 || self.byte_order == other.byte_order)
    }
}

impl std::str::FromStr for IntegerSpec {
    type Err = String;

    fn from_str(literal: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!("E_INT_TYPE: invalid integer type {literal:?}; expected e.g. u8, u32, or i64be")
        };
        let (number, byte_order) = if let Some(number) = literal.strip_suffix("be") {
            (number, ByteOrder::Big)
        } else {
            (literal, ByteOrder::Little)
        };
        let (signed, width) = match number.as_bytes().first() {
            Some(b'u') => (false, &number[1..]),
            Some(b'i') => (true, &number[1..]),
            _ => return Err(invalid()),
        };
        if width.is_empty()
            || width.starts_with('0')
            || !width.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(invalid());
        }
        let bits: u16 = width.parse().map_err(|_| {
            format!(
                "E_INT_WIDTH: unsupported {}{width}",
                if signed { "i" } else { "u" }
            )
        })?;
        if byte_order == ByteOrder::Big && bits == 8 {
            return Err(invalid());
        }
        let spec = Self {
            bits,
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
