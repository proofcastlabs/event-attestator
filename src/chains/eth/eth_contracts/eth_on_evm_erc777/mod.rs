use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{
    chains::eth::{
        eth_constants::{ETH_ADDRESS_SIZE_IN_BYTES, ETH_WORD_SIZE_IN_BYTES},
        eth_traits::EthLogCompatible,
    },
    types::{Bytes, Result},
};

lazy_static! {
    pub static ref ERC_777_REDEEM_EVENT_TOPIC: EthHash = {
        EthHash::from_slice(
            &hex::decode("78e6c3f67f57c26578f2487b930b70d844bcc8dd8f4d629fb4af81252ab5aa65")
                .expect("✘ Invalid hex in `BTC_ON_ETH_REDEEM_EVENT_TOPIC`"),
        )
    };
}

#[derive(Debug, Clone, Constructor, Eq, PartialEq)]
pub struct EthOnEvmErc777RedeemEvent {
    pub value: U256,
    pub redeemer: EthAddress,
    pub underlying_asset_recipient: EthAddress,
    pub user_data: Bytes,
}

impl EthOnEvmErc777RedeemEvent {
    fn get_err_msg(field: &str) -> String {
        format!("Error getting `{}` from `EthOnEvmErc777RedeemEvent`!", field)
    }

    pub fn from_log<T: EthLogCompatible>(log: &T) -> Result<Self> {
        let tokens = eth_abi_decode(
            &vec![
                EthAbiParamType::Uint(256),
                EthAbiParamType::Address,
                EthAbiParamType::Bytes,
            ],
            &log.get_data(),
        )?;
        Ok(Self {
            redeemer: EthAddress::from_slice(
                &log.get_topics()[1][ETH_WORD_SIZE_IN_BYTES - ETH_ADDRESS_SIZE_IN_BYTES..],
            ),
            value: match tokens[0] {
                EthAbiToken::Uint(value) => Ok(value),
                _ => Err(Self::get_err_msg("value").to_string()),
            }?,
            underlying_asset_recipient: match tokens[1] {
                EthAbiToken::Address(value) => Ok(value),
                _ => Err(Self::get_err_msg("underlying_asset_recipient").to_string()),
            }?,
            user_data: match tokens[2] {
                EthAbiToken::Bytes(ref value) => Ok(value.clone()),
                _ => Err(Self::get_err_msg("user_data").to_string()),
            }?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_test_utils::get_sample_log_with_eth_on_evm_erc777_peg_out;

    #[test]
    fn should_decode_eth_on_evm_erc777_event_correctly() {
        let log = get_sample_log_with_eth_on_evm_erc777_peg_out();
        let result = EthOnEvmErc777RedeemEvent::from_log(&log).unwrap();
        let expected_result = EthOnEvmErc777RedeemEvent {
            value: U256::from_dec_str("666").unwrap(),
            redeemer: EthAddress::from_slice(&hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap()),
            underlying_asset_recipient: EthAddress::from_slice(
                &hex::decode("fEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap(),
            ),
            user_data: vec![0xde, 0xca, 0xff],
        };
        assert_eq!(result, expected_result);
    }
}
