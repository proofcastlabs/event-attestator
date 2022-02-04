use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_constants::ZERO_ETH_VALUE,
        eth_contracts::erc777::encode_erc777_mint_fxn_maybe_with_data,
        eth_crypto::{
            eth_private_key::EthPrivateKey as EvmPrivateKey,
            eth_transaction::{EthTransaction as EvmTransaction, EthTransactions as EvmTransactions},
        },
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos},
    metadata::metadata_traits::ToMetadata,
    traits::DatabaseInterface,
    types::Result,
};

impl Erc20OnIntIntTxInfo {
    pub fn to_int_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransaction> {
        let operator_data = None;
        let metadata_bytes = self.to_metadata_bytes()?;
        info!("✔ Signing INT transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata_bytes));
        encode_erc777_mint_fxn_maybe_with_data(
            &self.router_address,
            &self.get_host_token_amount(dictionary)?,
            Some(metadata_bytes),
            operator_data,
        )
        .map(|data| {
            EvmTransaction::new_unsigned(
                data,
                nonce,
                ZERO_ETH_VALUE,
                self.evm_token_address,
                chain_id,
                gas_limit,
                gas_price,
            )
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(evm_private_key))
    }
}

impl Erc20OnIntIntTxInfos {
    pub fn to_int_signed_txs(
        &self,
        start_nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransactions> {
        info!("✔ Signing `erc20-on-int` INT transactions...");
        Ok(EvmTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    Erc20OnIntIntTxInfo::to_int_signed_tx(
                        tx_info,
                        start_nonce + i as u64,
                        chain_id,
                        gas_limit,
                        gas_price,
                        evm_private_key,
                        dictionary,
                    )
                })
                .collect::<Result<Vec<EvmTransaction>>>()?,
        ))
    }
}

pub fn maybe_sign_int_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.erc20_on_int_int_tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no INT transactions to sign!");
        Ok(state)
    } else {
        let chain_id = state.evm_db_utils.get_eth_chain_id_from_db()?;
        state
            .erc20_on_int_int_tx_infos
            .to_int_signed_txs(
                state.evm_db_utils.get_eth_account_nonce_from_db()?,
                &chain_id,
                chain_id.get_erc777_mint_with_data_gas_limit(),
                state.evm_db_utils.get_eth_gas_price_from_db()?,
                &state.evm_db_utils.get_eth_private_key_from_db()?,
                &EthEvmTokenDictionary::get_from_db(state.db)?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_erc20_on_int_int_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_traits::EthTxInfoCompatible,
        erc20_on_int::test_utils::{get_sample_evm_private_key, get_sample_int_tx_infos, get_sample_token_dictionary},
    };

    #[test]
    fn should_get_signaures_from_evm_tx_info() {
        let dictionary = get_sample_token_dictionary();
        let pk = get_sample_evm_private_key();
        let infos = get_sample_int_tx_infos();
        let nonce = 0_u64;
        let chain_id = EthChainId::Rinkeby;
        let gas_limit = 300_000_usize;
        let gas_price = 20_000_000_000_u64;
        let signed_txs = infos
            .to_int_signed_txs(nonce, &chain_id, gas_limit, gas_price, &pk, &dictionary)
            .unwrap();
        let expected_num_results = 1;
        assert_eq!(signed_txs.len(), expected_num_results);
        let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
        let expected_tx_hex = "f902ab808504a817c800830493e094a83446f219baec0b6fd6b3031c5a49a54543045b80b90244dcdc7dd00000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f600000000000000000000000000000000000000000000000000000000000005390000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000001800200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000f3436800000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac0069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003c0ffee00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ba0d34861a7beb7dfec45003ff4477954e9a6a66e68ceca964a01a223512b7773e2a030cacf4f50400efbec17bc080115e1aee3d8df0afe1b8baff25a48860564f90f";
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
