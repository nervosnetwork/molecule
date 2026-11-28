#![allow(clippy::upper_case_acronyms)]

use std::{convert::TryFrom, fmt, io, str};

use super::ToIntermediate;
use crate::ast;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    JSON,
    YAML,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::JSON => write!(f, "JSON"),
            Self::YAML => write!(f, "YAML"),
        }
    }
}

impl TryFrom<&str> for Format {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "JSON" => Ok(Self::JSON),
            "YAML" => Ok(Self::YAML),
            format => Err(format!("unsupported format: [{}]", format)),
        }
    }
}

impl Format {
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::JSON => "json",
            Self::YAML => "yaml",
        }
    }

    fn serialize_value<T: serde::Serialize>(self, value: &T) -> Result<Vec<u8>, String> {
        let mut s = match self {
            Self::JSON => serde_json::to_string_pretty(value)
                .map_err(|err| format!("failed to serialize {}: {}", self, err))?,
            Self::YAML => serde_yaml::to_string(value)
                .map_err(|err| format!("failed to serialize {}: {}", self, err))?,
        };
        s.push('\n');
        Ok(s.into_bytes())
    }

    pub(crate) fn serialize(self, ir: &super::Ir) -> Result<Vec<u8>, String> {
        self.serialize_value(ir)
    }

    fn deserialize_str<T: serde::de::DeserializeOwned>(self, s: &str) -> Result<T, String> {
        match self {
            Self::JSON => serde_json::from_str(s).map_err(|err| err.to_string()),
            Self::YAML => serde_yaml::from_str(s).map_err(|err| err.to_string()),
        }
    }

    pub(crate) fn generate<W: io::Write>(self, writer: &mut W, ir: &super::Ir) -> io::Result<()> {
        let data = self
            .serialize(ir)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
        writer.write_all(&data)
    }

    pub fn recover(self, bytes: &[u8]) -> Result<ast::Ast, String> {
        let s = str::from_utf8(bytes).map_err(|err| format!("failed to convert bytes: {}", err))?;
        match self.deserialize_str::<super::Ir>(s) {
            Ok(ir) => ir.recover(false),
            Err(legacy_error) => {
                let envelope: super::Envelope = self.deserialize_str(s).map_err(|error| {
                    format!(
                        "failed to deserialize {}: {}; invalid v2 {}: {}",
                        self, legacy_error, self, error
                    )
                })?;
                envelope.recover()
            }
        }
    }

    /// Export v1 for plain layouts and v2 whenever explicit source semantics are present.
    pub fn serialize_ast(self, ast: &ast::Ast) -> Result<Vec<u8>, String> {
        if ast.source_graph().is_none() {
            return self.serialize(&ast.to_ir());
        }
        self.serialize_value(&super::Envelope::from_ast(ast)?)
    }

    /// Explicitly discard extensions and export only the legacy wire layout.
    pub fn serialize_wire(self, ast: &ast::Ast) -> Result<Vec<u8>, String> {
        self.serialize(&ast.to_ir())
    }
}
