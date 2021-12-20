use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_constants::ZERO_ETH_VALUE,
        eth_contracts::erc20_vault::{
            encode_erc20_vault_peg_out_fxn_data_with_user_data,
            ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
        },
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
        },
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
    },
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    metadata::metadata_traits::ToMetadata,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEvmIntTxInfo {
    pub fn to_eth_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        evm_private_key: &EthPrivateKey,
        vault_address: &EthAddress,
    ) -> Result<EvmTransaction> {
        info!("✔ Signing ETH transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas price: {}", gas_price);
        debug!(
            "✔ Signing tx to token recipient: {}",
            self.destination_address.to_string()
        );
        debug!(
            "✔ Signing tx for token address : {}",
            self.eth_token_address.to_string()
        );
        debug!(
            "✔ Signing tx for token amount: {}",
            self.native_token_amount.to_string()
        );
        debug!("✔ Signing tx for vault address: {}", vault_address.to_string());
        EvmTransaction::new_unsigned(
            encode_erc20_vault_peg_out_fxn_data_with_user_data(
                self.router_address,
                self.eth_token_address,
                self.native_token_amount,
                self.to_metadata_bytes()?,
            )?,
            nonce,
            ZERO_ETH_VALUE,
            *vault_address,
            chain_id,
            ERC20_VAULT_PEGOUT_WITH_USER_DATA_GAS_LIMIT,
            gas_price,
        )
        .sign(evm_private_key)
    }
}

impl IntOnEvmIntTxInfos {
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
                    IntOnEvmIntTxInfo::to_eth_signed_tx(
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
    if state.int_on_evm_int_tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        info!("✔ Signing transactions for `IntOnEvmIntTxInfos`...");
        state
            .int_on_evm_int_tx_infos
            .to_eth_signed_txs(
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                &state.eth_db_utils.get_eth_chain_id_from_db()?,
                state.eth_db_utils.get_eth_gas_price_from_db()?,
                &state.eth_db_utils.get_eth_private_key_from_db()?,
                &state.eth_db_utils.get_int_on_evm_smart_contract_address_from_db()?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_int_on_evm_int_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_traits::EthTxInfoCompatible,
        int_on_evm::test_utils::{
            get_sample_eth_private_key,
            get_sample_peg_out_submission_material,
            get_sample_router_address,
            get_sample_token_dictionary,
            get_sample_vault_address,
        },
    };

    fn get_sample_tx_infos() -> IntOnEvmIntTxInfos {
        let dictionary = get_sample_token_dictionary();
        let material = get_sample_peg_out_submission_material();
        let router_address = get_sample_router_address();
        IntOnEvmIntTxInfos::from_submission_material(&material, &dictionary, &router_address).unwrap()
    }

    #[test]
    fn should_get_signatures_from_eth_tx_info() {
        let infos = get_sample_tx_infos();
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
        let expected_tx_hex = "f9028b808504a817c8008306ddd094010e1e6f6c360da7e3d62479b6b9d717b3e114ca80b90224229654690000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6000000000000000000000000a83446f219baec0b6fd6b3031c5a49a54543045b000000000000000000000000000000000000000000000000000000000000029a00000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000180020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac00f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003decaff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ca076f2b4895359adff4cd879a104c886a1e4a08d057ec7303fc80d74acd9600870a04b7192be0b3e6a41ec08616b9bbaaf10967695b73e7749c13bc43523296457c3";
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
