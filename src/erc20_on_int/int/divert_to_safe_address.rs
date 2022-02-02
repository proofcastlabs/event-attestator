use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::eth_state::EthState,
    constants::SAFE_ETH_ADDRESS,
    erc20_on_int::int::eth_tx_info::{EthOnIntEthTxInfo, EthOnIntEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

create_diversion_fxns!("EthOnIntEthTxInfo" => "Eth" => "erc20_on_int_eth_tx_infos" => "vault", "token");

#[cfg(test)]
mod tests {
    use ethereum_types::{H256 as EthHash, U256};

    use super::*;
    use crate::chains::eth::{eth_chain_id::EthChainId, eth_utils::convert_hex_to_eth_address};

    #[test]
    fn should_divert_to_safe_address_if_destination_is_token_address() {
        let mut info = EthOnIntEthTxInfo::default();
        let eth_address = "0x89ab32156e46f46d02ade3fecbe5fc4243b9aaed";
        let destination_address = convert_hex_to_eth_address(eth_address).unwrap();
        info.eth_token_address = destination_address.clone();
        info.destination_address = destination_address.clone();
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_token_contract_address();
        assert_eq!(result.destination_address, *SAFE_ETH_ADDRESS);
    }

    #[test]
    fn should_divert_to_safe_address_if_destination_is_vault_address() {
        let mut info = EthOnIntEthTxInfo::default();
        let eth_address = "0x89ab32156e46f46d02ade3fecbe5fc4243b9aaed";
        let destination_address = convert_hex_to_eth_address(eth_address).unwrap();
        info.eth_vault_address = destination_address.clone();
        info.destination_address = destination_address.clone();
        assert_eq!(info.destination_address, destination_address);
        let result = info.divert_to_safe_address_if_destination_is_vault_contract_address();
        assert_eq!(result.destination_address, *SAFE_ETH_ADDRESS);
    }
}
