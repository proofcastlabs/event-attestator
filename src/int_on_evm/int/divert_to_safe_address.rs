use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_EVM_ADDRESS,
    int_on_evm::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEvmEvmTxInfo {
    fn update_destination_address(&self, new_address: EthAddress) -> Self {
        let mut new_self = self.clone();
        new_self.destination_address = new_address;
        new_self
    }

    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        info!("✔ Checking if the destination address is the same as the INT token contract address...");
        if self.destination_address == self.evm_token_address {
            info!("✔ Recipient address is same as INT token address! Diverting to safe address...");
            self.update_destination_address(*SAFE_EVM_ADDRESS)
        } else {
            self.clone()
        }
    }
}

impl IntOnEvmEvmTxInfos {
    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        Self::new(
            self.iter()
                .map(|info| info.divert_to_safe_address_if_destination_is_token_contract_address())
                .collect::<Vec<IntOnEvmEvmTxInfo>>(),
        )
    }
}

pub fn maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.int_on_evm_evm_tx_infos.is_empty() {
        Ok(state)
    } else {
        info!("✔ Maybe diverting EVM txs to safe address if destination address is the token contract address...");
        let new_infos = state
            .int_on_evm_evm_tx_infos
            .divert_to_safe_address_if_destination_is_token_contract_address();
        state.replace_int_on_evm_evm_tx_infos(new_infos)
    }
}

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::{int_on_evm::test_utils::get_sample_router_address, metadata::metadata_chain_id::MetadataChainId};

    #[test]
    fn should_divert_to_safe_address_if_destination_is_token_address() {
        let destination_address =
            EthAddress::from_slice(&hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap());
        let router_address = get_sample_router_address();
        let info = IntOnEvmEvmTxInfo {
            router_address,
            user_data: vec![],
            destination_address,
            native_token_amount: U256::from_dec_str("1000000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            eth_token_address: EthAddress::from_slice(
                &hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("578670d0e08ca172eb8e862352e731814564fd6a12c3143e88bfb28292cd1535").unwrap(),
            ),
            origin_chain_id: MetadataChainId::EthereumMainnet,
            destination_chain_id: MetadataChainId::BscMainnet,
        };
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_token_contract_address();
        assert_eq!(result.destination_address, *SAFE_EVM_ADDRESS);
    }
}
