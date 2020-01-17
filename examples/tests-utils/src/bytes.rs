use std::fmt;

use proc_macro2 as m4;

#[derive(Default)]
pub(crate) struct Bytes(Vec<u8>);

pub(crate) struct BytesVisitor;

impl fmt::Debug for Bytes {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = &self.0;
        write!(f, "&[")?;
        if data.len() != 0 {
            write!(f, "{:#04x}u8", data[0])?;
            for unit in data.iter().skip(1) {
                write!(f, ", {:#04x}", unit)?;
            }
        }
        write!(f, "]")
    }
}

impl<'b> serde::de::Visitor<'b> for BytesVisitor {
    type Value = Bytes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "require a 0x-prefixed hexadecimal string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() < 2 || &v.as_bytes()[0..2] != b"0x" {
            return Err(E::invalid_value(serde::de::Unexpected::Str(v), &self));
        }

        if v.len() == 2 {
            return Ok(Default::default());
        }

        let v_new = &v[2..].replace("_", "").replace("/", "");
        let bytes = v_new.as_bytes();
        if v_new.len() % 2 != 0 {
            return Err(E::invalid_value(serde::de::Unexpected::Str(v), &self));
        }
        let mut buffer = vec![0; bytes.len() >> 1];
        faster_hex::hex_decode(bytes, &mut buffer)
            .map_err(|e| E::custom(format_args!("{:?}", e)))?;
        Ok(Bytes(buffer))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }
}

impl<'de> serde::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(BytesVisitor)
    }
}

impl Bytes {
    pub(crate) fn ts(&self) -> m4::TokenStream {
        format!("{:?}", self).parse().unwrap_or_else(|_| {
            panic!("Failed to parse the string <{:?}> to TokenStream.", self);
        })
    }
}
