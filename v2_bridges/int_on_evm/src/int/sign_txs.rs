use common::{
    dictionaries::eth_evm::EthEvmTokenDictionary,
    metadata::metadata_traits::ToMetadata,
    safe_addresses::safely_convert_str_to_eth_address,
    traits::DatabaseInterface,
    types::Result,
    EthChainId,
};
use common_eth::{
    encode_erc777_mint_fxn_maybe_with_data,
    EthDbUtilsExt,
    EthPrivateKey as EvmPrivateKey,
    EthState,
    EthTransaction as EvmTransaction,
    EthTransactions as EvmTransactions,
    ZERO_ETH_VALUE,
};
use ethereum_types::U256;

use crate::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos};

impl IntOnEvmEvmTxInfo {
    pub fn get_host_token_amount(&self, dictionary: &EthEvmTokenDictionary) -> Result<U256> {
        dictionary.convert_eth_amount_to_evm_amount(&self.eth_token_address, self.native_token_amount)
    }

    pub fn to_evm_signed_tx(
        &self,
        nonce: u64,
        chain_id: &EthChainId,
        gas_limit: usize,
        gas_price: u64,
        evm_private_key: &EvmPrivateKey,
        dictionary: &EthEvmTokenDictionary,
    ) -> Result<EvmTransaction> {
        let operator_data = None;
        let destination_eth_address = safely_convert_str_to_eth_address(&self.destination_address);
        let metadata_bytes = if self.user_data.is_empty() {
            vec![]
        } else {
            self.to_metadata_bytes()?
        };
        info!("✔ Signing INT transaction for tx info: {:?}", self);
        debug!("✔ Signing with nonce:     {}", nonce);
        debug!("✔ Signing with chain id:  {}", chain_id);
        debug!("✔ Signing with gas limit: {}", gas_limit);
        debug!("✔ Signing with gas price: {}", gas_price);
        if !metadata_bytes.is_empty() {
            debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata_bytes))
        } else {
            debug!("✔ No user data ∴ not wrapping in metadata!");
        };
        encode_erc777_mint_fxn_maybe_with_data(
            &destination_eth_address,
            &self.get_host_token_amount(dictionary)?,
            if metadata_bytes.is_empty() {
                None
            } else {
                Some(metadata_bytes)
            },
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

impl IntOnEvmEvmTxInfos {
    pub fn to_evm_signed_txs(
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
                    IntOnEvmEvmTxInfo::to_evm_signed_tx(
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

pub fn maybe_sign_evm_txs_and_add_to_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ No tx infos in state ∴ no INT transactions to sign!");
        Ok(state)
    } else {
        let chain_id = state.evm_db_utils.get_eth_chain_id_from_db()?;
        IntOnEvmEvmTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                tx_infos.to_evm_signed_txs(
                    state.evm_db_utils.get_eth_account_nonce_from_db()?,
                    &chain_id,
                    chain_id.get_erc777_mint_with_data_gas_limit(),
                    state.evm_db_utils.get_eth_gas_price_from_db()?,
                    &state.evm_db_utils.get_eth_private_key_from_db()?,
                    &EthEvmTokenDictionary::get_from_db(state.db)?,
                )
            })
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_int_on_evm_evm_signed_txs(signed_txs)
            })
    }
}

#[cfg(test)]
mod tests {
    use common_eth::EthTxInfoCompatible;

    use super::*;
    use crate::test_utils::{get_sample_evm_private_key, get_sample_evm_tx_infos, get_sample_token_dictionary};

    #[test]
    fn should_get_signatures_from_evm_tx_info() {
        let dictionary = get_sample_token_dictionary();
        let pk = get_sample_evm_private_key();
        let infos = get_sample_evm_tx_infos();
        let nonce = 0_u64;
        let chain_id = EthChainId::Rinkeby;
        let gas_limit = 300_000_usize;
        let gas_price = 20_000_000_000_u64;
        let signed_txs = infos
            .to_evm_signed_txs(nonce, &chain_id, gas_limit, gas_price, &pk, &dictionary)
            .unwrap();
        let expected_num_results = 1;
        assert_eq!(signed_txs.len(), expected_num_results);
        let tx_hex = signed_txs[0].eth_tx_hex().unwrap();
        let expected_tx_hex = "f902ab808504a817c800830493e094dd9f905a34a6c507c7d68384985905cf5eb032e980b90244dcdc7dd0000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000053900000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000018002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100ffffffff000000000000000000000000000000000000000000000000000000000000000000000000000000000e1c8524b1d1891b201ffc7bb58a82c96f8fc4f60069c32200000000000000000000000000000000000000000000000000000000000000000000000000000000fedfe2616eb3661cb8fed2782f5f0cc91d59dcac000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000003c0ffee00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002ba0352bf595d0e951217a4dc438b12e6d83a74d1393b250637cb3022645f4405092a01ac93750c7431df963f9db304b044512f8cba56f38e5e247c9846277115c6af9"
;
        assert_eq!(tx_hex, expected_tx_hex);
    }
}
