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

    pub(crate) fn serialize(self, ir: &super::Ir) -> Result<Vec<u8>, String> {
        match self {
            Self::JSON => serde_json::to_string_pretty(ir)
                .map_err(|err| format!("failed to serialize {}: {}", self, err))
                .map(|mut s| {
                    s.push('\n');
                    s
                })
                .map(String::into_bytes),
            Self::YAML => serde_yaml::to_string(ir)
                .map_err(|err| format!("failed to serialize {}: {}", self, err))
                .map(|mut s| {
                    s.push('\n');
                    s
                })
                .map(String::into_bytes),
        }
    }

    pub(crate) fn deserialize(self, bytes: &[u8]) -> Result<super::Ir, String> {
        match self {
            Self::JSON | Self::YAML => {
                let s = str::from_utf8(bytes)
                    .map_err(|err| format!("failed to convert bytes: {}", err))?;
                match self {
                    Self::JSON => serde_json::from_str(s)
                        .map_err(|err| format!("failed to deserialize {}: {}", self, err)),
                    Self::YAML => serde_yaml::from_str(s)
                        .map_err(|err| format!("failed to deserialize {}: {}", self, err)),
                }
            }
        }
    }

    pub(crate) fn generate<W: io::Write>(self, writer: &mut W, ir: &super::Ir) -> io::Result<()> {
        let data = self.serialize(ir).unwrap();
        writer.write_all(&data)
    }

    pub fn recover(self, bytes: &[u8]) -> Result<ast::Ast, String> {
        match self.deserialize(bytes) {
            Ok(ir) => ir.recover(false),
            Err(legacy_error) => {
                let envelope: super::Envelope = match self {
                    Self::JSON => serde_json::from_slice(bytes)
                        .map_err(|error| format!("{legacy_error}; invalid v2 JSON: {error}"))?,
                    Self::YAML => serde_yaml::from_slice(bytes)
                        .map_err(|error| format!("{legacy_error}; invalid v2 YAML: {error}"))?,
                };
                envelope.recover()
            }
        }
    }

    /// Export v1 for plain layouts and v2 whenever explicit source semantics are present.
    pub fn serialize_ast(self, ast: &ast::Ast) -> Result<Vec<u8>, String> {
        if ast.source_graph().is_none() {
            return self.serialize(&ast.to_ir());
        }
        let envelope = super::Envelope::from_ast(ast)?;
        let mut output = match self {
            Self::JSON => {
                serde_json::to_string_pretty(&envelope).map_err(|error| error.to_string())?
            }
            Self::YAML => serde_yaml::to_string(&envelope).map_err(|error| error.to_string())?,
        };
        output.push('\n');
        Ok(output.into_bytes())
    }

    /// Explicitly discard extensions and export only the legacy wire layout.
    pub fn serialize_wire(self, ast: &ast::Ast) -> Result<Vec<u8>, String> {
        self.serialize(&ast.to_ir())
    }
}
