pub(crate) mod erc20_vault;
pub(crate) mod erc777_proxy;
pub(crate) mod erc777_token;

use ethabi::{Contract as EthContract, Token};
use ethereum_types::H256 as EthHash;

use crate::types::{Bytes, Result};

pub fn instantiate_contract_from_abi(abi: &str) -> Result<EthContract> {
    Ok(EthContract::load(abi.as_bytes())?)
}

pub fn encode_fxn_call(abi: &str, fxn_name: &str, param_tokens: &[Token]) -> Result<Bytes> {
    Ok(instantiate_contract_from_abi(abi)?
        .function(fxn_name)?
        .encode_input(param_tokens)?)
}

pub trait SupportedTopics: strum::IntoEnumIterator {
    fn to_bytes(&self) -> Bytes;

    fn from_bytes(bytes: Bytes) -> Result<Self>
    where
        Self: Sized,
        Self: Clone,
    {
        let result = Self::get_all()
            .iter()
            .zip(Self::get_all_as_bytes().iter())
            .filter(|(_, supported_topic_bytes)| bytes == supported_topic_bytes.to_vec())
            .map(|(supported_topic, _)| supported_topic)
            .cloned()
            .collect::<Vec<Self>>();
        if result.is_empty() {
            Err("Cannot get supported topic from bytes - unrecognized topic!".into())
        } else {
            Ok(result[0].clone())
        }
    }

    fn get_all() -> Vec<Self>
    where
        Self: Sized,
    {
        Self::iter().collect()
    }

    fn get_all_as_bytes() -> Vec<Bytes>
    where
        Self: Sized,
    {
        Self::get_all().iter().map(Self::to_bytes).collect()
    }

    fn from_topic(topic: &EthHash) -> Result<Self>
    where
        Self: Sized,
        Self: Clone,
    {
        Self::from_bytes(topic.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::eth_contracts::erc777_proxy::ERC777_PROXY_ABI;

    #[test]
    fn should_instantiate_pnetwork_contract_from_abi() {
        let result = instantiate_contract_from_abi(ERC777_PROXY_ABI);
        assert!(result.is_ok());
    }
}
