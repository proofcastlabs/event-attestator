use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS,
    erc20_on_int::int::eth_tx_info::{EthOnIntEthTxInfo, EthOnIntEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

macro_rules! create_diversion_fxns {
    ($struct_name:expr => $state_name:expr => $tx_infos_name:expr => $($contract_name:expr),*) => {
        paste! {
            impl [< $struct_name s>] {
                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        Self::new(
                            self.iter()
                                .map(|info| {
                                    info.[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]()
                                })
                                .collect::<Vec<[<$struct_name>]>>(),
                        )
                    }
                )*
            }

            impl [< $struct_name >] {
                fn update_destination_address(&self, new_address: EthAddress) -> Self {
                    let mut new_self = self.clone();
                    new_self.destination_address = new_address;
                    new_self
                }

                fn divert_to_safe_address_if_destination_matches_address(&self, address: &EthAddress) -> Self {
                    if self.destination_address == *address {
                        self.update_destination_address(*SAFE_ETH_ADDRESS)
                    } else {
                        self.clone()
                    }
                }

                $(
                    pub fn [<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>](&self) -> Self {
                        info!("✔ Checking if the destination address matches the {} contract address...", $contract_name);
                        self.divert_to_safe_address_if_destination_matches_address(&self.[<eth_ $contract_name _address>])
                    }
                )*
            }

            $(
                pub fn [<maybe_divert_txs_to_safe_address_if_destination_is_ $contract_name _address>]<D: DatabaseInterface>(
                    state: [< $state_name State>]<D>,
                ) -> Result<[< $state_name State>]<D>> {
                    if state.[< $tx_infos_name >].is_empty() {
                        Ok(state)
                    } else {
                        info!("✔ Maybe diverting txs to safe address if destination matches {} address...", $contract_name);
                        let new_infos = state
                            .[< $tx_infos_name >]
                            .[<divert_to_safe_address_if_destination_is_ $contract_name _contract_address>]();
                        state.[<replace_ $tx_infos_name>](new_infos)
                    }
                }
            )*
        }
    }
}

create_diversion_fxns!("EthOnIntEthTxInfo" => "Eth" => "erc20_on_int_eth_tx_infos" => "vault", "token");

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::chains::eth::eth_chain_id::EthChainId;

    #[test]
    fn should_divert_to_safe_address_if_destination_is_token_address() {
        let destination_address =
            EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let eth_vault_address = EthAddress::default();
        let eth_token_address = destination_address.clone();
        let info = EthOnIntEthTxInfo {
            eth_vault_address,
            eth_token_address,
            user_data: vec![],
            destination_address,
            origin_chain_id: EthChainId::BscMainnet,
            native_token_amount: U256::from_dec_str("100000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("52c620012a6e278d56f582eb1dcb9241c9b2d14d7edc5dab15473b579ce2d2ea").unwrap(),
            ),
        };
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_token_contract_address();
        assert_eq!(result.destination_address, *SAFE_ETH_ADDRESS);
    }

    #[test]
    fn should_divert_to_safe_address_if_destination_is_vault_address() {
        let eth_vault_address =
            EthAddress::from_slice(&hex::decode("89ab32156e46f46d02ade3fecbe5fc4243b9aaed").unwrap());
        let destination_address = eth_vault_address.clone();
        let eth_token_address = eth_vault_address.clone();
        let info = EthOnIntEthTxInfo {
            eth_vault_address,
            eth_token_address,
            user_data: vec![],
            destination_address,
            origin_chain_id: EthChainId::BscMainnet,
            native_token_amount: U256::from_dec_str("100000000000000000").unwrap(),
            token_sender: EthAddress::from_slice(&hex::decode("8127192c2e4703dfb47f087883cc3120fe061cb8").unwrap()),
            evm_token_address: EthAddress::from_slice(
                &hex::decode("daacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92").unwrap(),
            ),
            originating_tx_hash: EthHash::from_slice(
                &hex::decode("52c620012a6e278d56f582eb1dcb9241c9b2d14d7edc5dab15473b579ce2d2ea").unwrap(),
            ),
        };
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_vault_contract_address();
        assert_eq!(result.destination_address, *SAFE_ETH_ADDRESS);
    }
}
