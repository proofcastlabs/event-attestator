use common::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_constants::ZERO_ETH_VALUE,
        eth_contracts::erc20_vault::encode_erc20_vault_peg_out_fxn_data_with_user_data,
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
        },
        eth_database_utils::EthDbUtilsExt,
        eth_utils::convert_eth_address_to_string,
        EthState,
    },
    metadata::metadata_traits::ToMetadata,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::Address as EthAddress;

use crate::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos};

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
            convert_eth_address_to_string(&self.eth_token_address),
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
            chain_id.get_erc20_vault_pegout_with_user_data_gas_limit(),
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
    if state.tx_infos.is_empty() {
        warn!("✘ No tx infos in state ∴ no ETH transactions to sign!");
        Ok(state)
    } else {
        info!("✔ Signing transactions for `IntOnEvmIntTxInfos`...");
        IntOnEvmIntTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                tx_infos.to_eth_signed_txs(
                    state.eth_db_utils.get_eth_account_nonce_from_db()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    state.eth_db_utils.get_eth_gas_price_from_db()?,
                    &state.eth_db_utils.get_eth_private_key_from_db()?,
                    &state.eth_db_utils.get_int_on_evm_smart_contract_address_from_db()?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_int_on_evm_int_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use common::chains::eth::eth_traits::EthTxInfoCompatible;

    use super::*;
    use crate::test_utils::{
        get_sample_eth_private_key,
        get_sample_peg_out_submission_material,
        get_sample_router_address,
        get_sample_token_dictionary,
        get_sample_vault_address,
    };

    fn get_sample_tx_infos() -> IntOnEvmIntTxInfos {
        let dictionary = get_sample_token_dictionary();
        let material = get_sample_peg_out_submission_material();
        let router_address = get_sample_router_address();
        let vault_address = EthAddress::default();
        IntOnEvmIntTxInfos::from_submission_material(&material, &dictionary, &router_address, &vault_address).unwrap()
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
        let expected_tx_hex = "f9034b808504a817c8008306ddd094010e1e6f6c360da7e3d62479b6b9d717b3e114ca80b902e4229654690000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6000000000000000000000000a83446f219baec0b6fd6b3031c5a49a54543045b000000000000000000000000000000000000000000000000000000000000029a00000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000240030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014000f343680000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002200000000000000000000000000000000000000000000000000000000000000003decaff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a30786665646665323631366562333636316362386665643237383266356630636339316435396463616300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ca0a01eeed08ae67473a796e2a4f5c51a2b11e901d44cecd4002f6a6e7e87921100a00bd04c73e55d2d7cf559fb557435ab37247a7cb813bc709c8dbb7a99cb01e597";
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
