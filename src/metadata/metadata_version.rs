use crate::types::{Byte, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataVersion {
    V1,
}

impl MetadataVersion {
    pub fn as_byte(&self) -> Byte {
        match self {
            MetadataVersion::V1 => 0x01,
        }
    }

    pub fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            1u8 => Ok(MetadataVersion::V1),
            _ => Err(format!("✘ Unrecognized version byte for `MetadataVersion`: {:?}", byte).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_metadata_version_bytes_roundtrip() {
        let metadata_version = MetadataVersion::V1;
        let byte = metadata_version.as_byte();
        let result = MetadataVersion::from_byte(&byte).unwrap();
        assert_eq!(result, metadata_version);
    }
}
