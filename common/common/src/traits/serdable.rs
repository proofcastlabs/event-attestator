use crate::types::{Byte, Bytes, Result};

pub trait Serdable {
    fn from_bytes(bytes: &[Byte]) -> Result<Self>
    where
        Self: Default,
        Self: for<'a> serde::Deserialize<'a>,
    {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }

    fn to_bytes(&self) -> Result<Bytes>
    where
        Self: Default,
        Self: serde::Serialize,
    {
        Ok(serde_json::to_vec(self)?)
    }
}
