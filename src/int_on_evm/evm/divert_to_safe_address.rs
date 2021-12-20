use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS,
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEvmIntTxInfo {
    fn update_destination_address(&self, new_address: EthAddress) -> Self {
        let mut new_self = self.clone();
        new_self.destination_address = new_address;
        new_self
    }

    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        info!("✔ Checking if the destination address is the same as the ETH token contract address...");
        if self.destination_address == self.eth_token_address {
            info!("✔ Recipient address is same as ETH token address! Diverting to safe address...");
            self.update_destination_address(*SAFE_ETH_ADDRESS)
        } else {
            self.clone()
        }
    }
}

impl IntOnEvmIntTxInfos {
    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        Self::new(
            self.iter()
                .map(|info| info.divert_to_safe_address_if_destination_is_token_contract_address())
                .collect::<Vec<IntOnEvmIntTxInfo>>(),
        )
    }
}

pub fn maybe_divert_txs_to_safe_address_if_destination_is_eth_token_address<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.int_on_evm_int_tx_infos.is_empty() {
        info!("✔ No transactions in state so skipping diversion check!");
        Ok(state)
    } else {
        info!("✔ Maybe diverting ETH txs to safe address if destination address is the token contract address...");
        let new_infos = state
            .int_on_evm_int_tx_infos
            .divert_to_safe_address_if_destination_is_token_contract_address();
        state.replace_int_on_evm_int_tx_infos(new_infos)
    }
}

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::{
        chains::eth::eth_utils::convert_hex_to_eth_address,
        int_on_evm::test_utils::get_sample_router_address,
        metadata::metadata_chain_id::MetadataChainId,
    };

    #[test]
    fn should_divert_to_safe_address_if_destination_is_token_address() {
        let destination_address = convert_hex_to_eth_address("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap();
        let destination_chain_id = MetadataChainId::EthereumRopsten;
        let router_address = get_sample_router_address();
        let info = IntOnEvmIntTxInfo {
            router_address,
            user_data: vec![],
            destination_address,
            destination_chain_id,
            origin_chain_id: MetadataChainId::BscMainnet,
            native_token_amount: U256::from_dec_str("100000000000000000").unwrap(),
            token_sender: convert_hex_to_eth_address("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap(),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            eth_token_address: EthAddress::from_slice(
                &hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("52c620012a6e278d56f582eb1dcb9241c9b2d14d7edc5dab15473b579ce2d2ea").unwrap(),
            ),
        };
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_token_contract_address();
        assert_eq!(result.destination_address, *SAFE_ETH_ADDRESS);
    }
}
