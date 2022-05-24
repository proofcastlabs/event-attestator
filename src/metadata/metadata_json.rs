use serde::{Deserialize, Serialize};

use crate::{
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_version::MetadataVersion,
        Metadata,
    },
    types::{Bytes, Result},
};

impl Metadata {
    pub fn to_json(&self) -> Result<MetadataJson> {
        info!("✔ Converting metadata to json...");
        if self.version == MetadataVersion::V1 || self.version == MetadataVersion::V2 {
            return Err(format!("Cannot convert metadata {} into json!", self.version).into());
        }
        Ok(MetadataJson {
            user_data: hex::encode(&self.user_data),
            version: hex::encode(self.version.to_bytes()),
            origin_address: self.origin_address.to_string(),
            destination_address: match &self.destination_address {
                Some(address) => Ok(address.to_string()),
                None => Err("No `destination_address` in metadata!"),
            }?,
            origin_chain_id: hex::encode(self.origin_chain_id.to_bytes()?),
            destination_chain_id: match self.destination_chain_id {
                Some(id) => Ok(hex::encode(id.to_bytes()?)),
                None => Err("Non`destination_chain_id` in metadata!"),
            }?,
            protocol_options: match &self.protocol_options {
                None => None,
                Some(options) => Some(hex::encode(options)),
            },
            protocol_receipt: match &self.protocol_receipt {
                None => None,
                Some(receipt) => Some(hex::encode(receipt)),
            },
        })
    }

    #[cfg(test)]
    pub fn from_json(json: &MetadataJson) -> Result<Self> {
        let origin_chain_id = MetadataChainId::from_bytes(&hex::decode(&json.origin_chain_id)?)?;
        let destination_chain_id = MetadataChainId::from_bytes(&hex::decode(&json.destination_chain_id)?)?;
        let origin_address = MetadataAddress::new(&json.origin_address, &origin_chain_id)?;
        let destination_address = MetadataAddress::new(&json.destination_address, &destination_chain_id)?;
        // FIXME Do we need to check if they've defaulted to the safe address?
        Ok(Self {
            origin_address,
            origin_chain_id,
            user_data: hex::decode(&json.user_data)?,
            destination_address: Some(destination_address),
            destination_chain_id: Some(destination_chain_id),
            version: MetadataVersion::from_bytes(&hex::decode(&json.version)?)?,
            protocol_options: match &json.protocol_options {
                Some(options_hex) => Some(hex::decode(&options_hex)?),
                None => None,
            },
            protocol_receipt: match &json.protocol_receipt {
                Some(receipt_hex) => Some(hex::decode(&receipt_hex)?),
                None => None,
            },
        })
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataJson {
    pub version: String,
    pub user_data: String,
    pub origin_address: String,
    pub origin_chain_id: String,
    pub destination_address: String,
    pub destination_chain_id: String,
    pub protocol_options: Option<String>,
    pub protocol_receipt: Option<String>,
}

impl MetadataJson {
    pub fn to_bytes() -> Bytes {
        unimplemented!()
    }

    pub fn from_bytes() -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{errors::AppError, metadata::test_utils::get_sample_eth_metadata_v3};

    #[test]
    fn should_serde_metadata_to_and_from_json() {
        let metadata = get_sample_eth_metadata_v3();
        let json = metadata.to_json().unwrap();
        let result = Metadata::from_json(&json).unwrap();
        assert_eq!(result, metadata);
    }

    #[test]
    fn should_err_for_v2_metadata() {
        let mut metadata = Metadata::default();
        metadata.version = MetadataVersion::V2;
        let expected_error = format!("Cannot convert metadata {} into json!", metadata.version);
        match metadata.to_json() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
        println!("{:?}", metadata)
    }

    #[test]
    fn should_err_for_v1_metadata() {
        let mut metadata = Metadata::default();
        metadata.version = MetadataVersion::V1;
        let expected_error = format!("Cannot convert metadata {} into json!", metadata.version);
        match metadata.to_json() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
        println!("{:?}", metadata)
    }
}
