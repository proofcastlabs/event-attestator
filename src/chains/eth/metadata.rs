use std::fmt;
use crate::types::Byte;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EthMetadataVersion {
    V1,
}

impl EthMetadataVersion {
    pub fn get_version_byte(&self) -> Byte {
        match self {
            EthMetadataVersion::V1 => 0x01,
        }
    }
}

impl fmt::Display for EthMetadataVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EthMetadataVersion::V1 => write!(f, "1"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_metadata_v1_byte() {
        let expected_result = 0x01;
        let metadata_version = EthMetadataVersion::V1;
        let result = metadata_version.get_version_byte();
        assert_eq!(result ,expected_result);
    }
}
