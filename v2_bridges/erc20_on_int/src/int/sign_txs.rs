use common::{
    metadata::metadata_traits::ToMetadata,
    safe_addresses::safely_convert_str_to_eth_address,
    traits::DatabaseInterface,
    types::Result,
    EthChainId,
};
use common_eth::{
    encode_erc20_vault_peg_out_fxn_data_with_user_data,
    encode_erc20_vault_peg_out_fxn_data_without_user_data,
    EthDbUtilsExt,
    EthPrivateKey,
    EthState,
    EthTransaction as EvmTransaction,
    EthTransactions as EvmTransactions,
    ZERO_ETH_VALUE,
};
use ethereum_types::Address as EthAddress;

use crate::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos};

impl Erc20OnIntEthTxInfo {
    pub fn to_eth_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransaction> {
        let destination_eth_address = safely_convert_str_to_eth_address(&self.destination_address);

        let gas_limit = if self.user_data.is_empty() {
            chain_id.get_erc20_vault_pegout_without_user_data_gas_limit()
        } else {
            chain_id.get_erc20_vault_pegout_with_user_data_gas_limit()
        };

        info!("✔ Signing ETH transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        debug!("✔ Signing tx to token recipient: {}", destination_eth_address);
        debug!(
            "✔ Signing tx for token address : {}",
            self.eth_token_address.to_string()
        );
        debug!(
            "✔ Signing tx for token amount: {}",
            self.native_token_amount.to_string()
        );
        debug!("✔ Signing tx for vault address: {}", vault_address.to_string());
        let encoded_tx_data = if self.user_data.is_empty() {
            encode_erc20_vault_peg_out_fxn_data_without_user_data(
                destination_eth_address,
                self.eth_token_address,
                self.native_token_amount,
            )?
        } else {
            encode_erc20_vault_peg_out_fxn_data_with_user_data(
                destination_eth_address,
                self.eth_token_address,
                self.native_token_amount,
                self.to_metadata_bytes()?,
            )?
        };
        EvmTransaction::new_unsigned(
            encoded_tx_data,
            nonce,
            ZERO_ETH_VALUE,
            *vault_address,
            chain_id,
            gas_limit,
            gas_price,
        )
        .sign(evm_private_key)
    }
}

impl Erc20OnIntEthTxInfos {
    pub fn to_eth_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `ERC20-on-EVM` ETH transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    Erc20OnIntEthTxInfo::to_eth_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_price,
                        evm_private_key,
                        vault_address,
                    )
                })
                .collect::<Result<Vec<EvmTransaction>>>()?,
        ))
    }
}

pub fn maybe_sign_eth_txs_and_add_to_evm_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        Erc20OnIntEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                tx_infos.to_eth_signed_txs(
                    state.eth_db_utils.get_eth_account_nonce_from_db()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    state.eth_db_utils.get_eth_gas_price_from_db()?,
                    &state.eth_db_utils.get_eth_private_key_from_db()?,
                    &state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_erc20_on_int_eth_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use common_eth::EthTxInfoCompatible;

    use super::*;
    use crate::test_utils::{get_sample_eth_private_key, get_sample_eth_tx_infos, get_sample_vault_address};
    #[test]
    fn should_get_signatures_from_eth_tx_info() {
        let infos = get_sample_eth_tx_infos();
        let vault_address = get_sample_vault_address();
        let pk = get_sample_eth_private_key();
        let nonce = 0_u64;
        let chain_id = EthChainId::Rinkeby;
        let gas_price = 20_000_000_000_u64;
        let signed_txs = infos
            .to_eth_signed_txs(nonce, &chain_id, gas_price, &pk, &vault_address)
            .unwrap();
        let expected_num_results = 1;
        assert_eq!(signed_txs.len(), expected_num_results);
        let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
        let expected_tx_hex ="f901cb808504a817c8008306ddd094866e3fc7043efb8ff3a994f7d59f53fe045d4d7a80b9016422965469000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000c63ab9437f5589e2c67e04c00a98506b431276450000000000000000000000000000000000000000000000000000000000000299000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800069c322000000000000000000000000000000000000000000000000000000000000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60000000000000000000000000000000000000000000000000000000000000003decaff00000000000000000000000000000000000000000000000000000000002ca05d947b2392e1f828dfe1542ec1340a9926f5a6df1511809c4ae9f6b553d144a4a0484c711f8a39a1858dacb779e56bbdc95de00367db633d5edea1e0aabff67f4e";
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
