use crate::{
    btc_on_eth::utils::convert_satoshis_to_ptoken,
    types::{
        Byte,
        Bytes,
        Result,
    },
    chains::{
        eth::eth_utils::safely_convert_hex_to_eth_address,
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
    },
};
use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
};
use ethereum_types::{
    U256,
    Address as EthAddress,
};

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnEthMintingParams(pub Vec<BtcOnEthMintingParamStruct>);

impl BtcOnEthMintingParams {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn filter_out_value_too_low(&self) -> Result<BtcOnEthMintingParams> {
        info!("✔ Filtering out any minting params below a minimum of {} Satoshis...", MINIMUM_REQUIRED_SATOSHIS);
        let threshold = convert_satoshis_to_ptoken(MINIMUM_REQUIRED_SATOSHIS);
        Ok(BtcOnEthMintingParams::new(
            self
                .iter()
                .filter(|params| {
                    match params.amount >= threshold {
                        true => true,
                        false => {
                            info!("✘ Filtering minting params ∵ value too low: {:?}", params);
                            false
                        }
                    }
                })
                .cloned()
                .collect::<Vec<BtcOnEthMintingParamStruct>>()
        ))
    }

}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEthMintingParamStruct {
    pub amount: U256,
    pub eth_address: EthAddress,
    pub originating_tx_hash: sha256d::Hash,
    pub originating_tx_address: String,
}

impl BtcOnEthMintingParamStruct {
    pub fn new(
        amount: U256,
        eth_address_hex: String,
        originating_tx_hash: sha256d::Hash,
        originating_tx_address: BtcAddress,
    ) -> Result<BtcOnEthMintingParamStruct> {
        Ok(BtcOnEthMintingParamStruct {
            amount,
            originating_tx_hash,
            originating_tx_address: originating_tx_address.to_string(),
            eth_address: safely_convert_hex_to_eth_address(&eth_address_hex)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::btc::btc_test_utils::get_sample_minting_params;

    #[test]
    fn should_filter_minting_params() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let minting_params = get_sample_minting_params();
        let threshold = convert_satoshis_to_ptoken(MINIMUM_REQUIRED_SATOSHIS);
        let length_before = minting_params.len();
        assert_eq!(length_before, expected_length_before);
        let result = minting_params.filter_out_value_too_low().unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result.iter().map(|params| assert!(params.amount >= threshold)).for_each(drop);
    }
}
