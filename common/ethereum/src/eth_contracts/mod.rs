mod erc20_token;
mod erc20_vault;
mod erc777_proxy;
mod erc777_token;
mod weth;

pub use self::{
    erc20_token::{Erc20TokenTransferEvent, Erc20TokenTransferEvents, ToErc20TokenTransferEvent},
    erc20_vault::{
        encode_erc20_vault_add_supported_token_fx_data,
        encode_erc20_vault_migrate_fxn_data,
        encode_erc20_vault_migrate_single_fxn_data,
        encode_erc20_vault_peg_out_fxn_data_with_user_data,
        encode_erc20_vault_peg_out_fxn_data_without_user_data,
        encode_erc20_vault_remove_supported_token_fx_data,
        encode_erc20_vault_set_weth_unwrapper_address_fxn_data,
        Erc20VaultPegInEventParams,
        ERC20_VAULT_PEG_IN_EVENT_TOPIC_V2,
        ERC20_VAULT_PEG_IN_EVENT_WITHOUT_USER_DATA_TOPIC,
        ERC20_VAULT_PEG_IN_EVENT_WITH_USER_DATA_TOPIC,
    },
    erc777_proxy::{
        encode_mint_by_proxy_tx_data,
        get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
        get_signed_erc777_proxy_change_pnetwork_tx,
    },
    erc777_token::{
        encode_erc777_mint_fxn_maybe_with_data,
        encode_erc777_mint_with_no_data_fxn,
        get_signed_erc777_change_pnetwork_tx,
        Erc777BurnEvent,
        Erc777RedeemEvent,
        ERC777_REDEEM_EVENT_TOPIC_V2,
        ERC_777_BURN_EVENT_TOPIC,
        ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA,
        ERC_777_REDEEM_EVENT_TOPIC_WITH_USER_DATA,
    },
    weth::WethDepositEvents,
};

#[cfg(test)]
mod test_utils;

use common::types::{Bytes, Result};
use ethabi::{Contract as EthContract, Token};
use ethereum_types::H256 as EthHash;

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
    use crate::eth_contracts::erc777_proxy::ERC777_PROXY_ABI;

    #[test]
    fn should_instantiate_pnetwork_contract_from_abi() {
        let result = instantiate_contract_from_abi(ERC777_PROXY_ABI);
        assert!(result.is_ok());
    }
}
