use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS,
    erc20_on_int::int::eth_tx_info::{EthOnIntEthTxInfo, EthOnIntEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl EthOnIntEthTxInfo {
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

impl EthOnIntEthTxInfos {
    pub fn divert_to_safe_address_if_destination_is_token_contract_address(&self) -> Self {
        Self::new(
            self.iter()
                .map(|info| info.divert_to_safe_address_if_destination_is_token_contract_address())
                .collect::<Vec<EthOnIntEthTxInfo>>(),
        )
    }
}

pub fn maybe_divert_txs_to_safe_address_if_destination_is_eth_token_address<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.erc20_on_int_eth_tx_infos.is_empty() {
        Ok(state)
    } else {
        info!("✔ Maybe diverting ETH txs to safe address if destination address is the token contract address...");
        let new_infos = state
            .erc20_on_int_eth_tx_infos
            .divert_to_safe_address_if_destination_is_token_contract_address();
        state.replace_erc20_on_int_eth_tx_infos(new_infos)
    }
}

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::chains::eth::eth_chain_id::EthChainId;

    #[test]
    fn should_divert_to_safe_address_if_destination_is_token_address() {
        let destination_address =
            EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let info = EthOnIntEthTxInfo {
            user_data: vec![],
            destination_address,
            origin_chain_id: EthChainId::BscMainnet,
            native_token_amount: U256::from_dec_str("100000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
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
