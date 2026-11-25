use crate::{
    ast::Ast,
    integer::{ByteOrder, IntegerSpec},
    Language,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub capabilities_version: u32,
    pub ir_versions: Vec<u32>,
    pub formats: Vec<String>,
    pub features: Vec<String>,
    pub api_flavor: String,
    pub integers: Vec<IntegerSpec>,
}

impl Capabilities {
    pub fn builtin(language: Language, format: super::Format) -> Self {
        let mut integers = Vec::new();
        for (bits, signed) in IntegerSpec::SUPPORTED_KINDS {
            integers.push(IntegerSpec {
                bits,
                signed,
                byte_order: ByteOrder::Little,
            });
            if bits > 8 {
                integers.push(IntegerSpec {
                    bits,
                    signed,
                    byte_order: ByteOrder::Big,
                });
            }
        }
        Self {
            capabilities_version: 1,
            ir_versions: vec![1, 2],
            formats: vec![format.to_string().to_ascii_lowercase()],
            features: vec![super::INTEGER_FEATURE.into()],
            api_flavor: match language {
                Language::Rust => "rust-entity",
                Language::RustLazyReader => "rust-lazy",
                Language::C => "c",
            }
            .into(),
            integers,
        }
    }

    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes)
            .map_err(|error| format!("E_PLUGIN_CAPABILITY: invalid response: {error}"))
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| format!("E_PLUGIN_CAPABILITY: {error}"))
    }

    pub fn require(&self, ast: &Ast) -> Result<super::Format, String> {
        if self.capabilities_version != 1 || !self.ir_versions.contains(&2) {
            return Err(
                "E_PLUGIN_CAPABILITY: plugin must support capability protocol 1 and IR 2".into(),
            );
        }
        if !ast.integers().is_empty()
            && !self
                .features
                .iter()
                .any(|feature| feature == super::INTEGER_FEATURE)
        {
            return Err("E_PLUGIN_CAPABILITY: plugin does not support logical-integer-v1".into());
        }
        for (name, spec) in ast.integers() {
            let canonical = spec.canonical();
            if !self.integers.contains(&canonical) {
                return Err(format!(
                    "E_PLUGIN_CAPABILITY: plugin cannot represent {name}: {canonical:?}"
                ));
            }
        }
        self.formats
            .iter()
            .find_map(|format| super::Format::try_from(format.as_str()).ok())
            .ok_or_else(|| "E_PLUGIN_CAPABILITY: no supported IR transport format".into())
    }
}
