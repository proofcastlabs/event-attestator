use crate::types::{Byte, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataVersion {
    V1,
}

impl MetadataVersion {
    pub fn as_byte(&self) -> Byte {
        match self {
            Self::V1 => 0x01,
        }
    }

    pub fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            1u8 => Ok(Self::V1),
            _ => Err(format!("✘ Unrecognized version byte for `MetadataVersion`: {:?}", byte).into()),
        }
    }

    #[cfg(test)]
    fn get_all() -> Vec<Self> {
        // TODO How to ensure this list is complete?
        vec![Self::V1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_metadata_version_bytes_roundtrip() {
        MetadataVersion::get_all().iter().for_each(|id| {
            let byte = id.as_byte();
            let result = MetadataVersion::from_byte(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }
}
