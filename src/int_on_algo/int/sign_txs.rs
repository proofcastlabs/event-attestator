use rust_algorand::{
    AlgorandAddress,
    AlgorandHash,
    AlgorandKeys,
    AlgorandSignedTransaction,
    AlgorandTransaction,
    MicroAlgos,
};

use crate::{
    chains::eth::eth_state::EthState,
    int_on_algo::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos},
    metadata::metadata_traits::ToMetadata,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnAlgoAlgoTxInfo {
    pub fn to_algo_signed_tx(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        private_key: &AlgorandKeys,
    ) -> Result<AlgorandSignedTransaction> {
        info!("✔ Signing ALGO transaction for tx info: {:?}", self);
        let metadata_bytes = if self.user_data.is_empty() {
            vec![]
        } else {
            self.to_metadata_bytes()?
        };
        let metadata = if metadata_bytes.is_empty() {
            debug!("✔ No user data ∴ not wrapping in metadata!");
            None
        } else {
            debug!("✔ Signing with metadata : 0x{}", hex::encode(&metadata_bytes));
            Some(metadata_bytes)
        };
        let last_valid = None;
        Ok(AlgorandTransaction::asset_transfer(
            self.algo_asset_id,
            *fee,
            self.host_token_amount.as_u64(),
            metadata,
            first_valid,
            *sender,
            genesis_hash.clone(),
            last_valid,
            self.destination_address,
        )
        .and_then(|tx| tx.sign(private_key))?)
    }
}

impl IntOnAlgoAlgoTxInfos {
    pub fn to_algo_signed_txs(
        &self,
        fee: &MicroAlgos,
        first_valid: u64,
        genesis_hash: &AlgorandHash,
        sender: &AlgorandAddress,
        private_key: &AlgorandKeys,
    ) -> Result<Vec<AlgorandSignedTransaction>> {
        info!("✔ Signing `erc20-on-int` INT transactions...");
        self.iter()
            .enumerate()
            .map(|(i, info)| info.to_algo_signed_tx(fee, first_valid + i as u64, genesis_hash, sender, private_key))
            .collect::<Result<Vec<_>>>()
    }
}

pub fn maybe_sign_algo_txs_and_add_to_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    let tx_infos = state.int_on_algo_algo_tx_infos.clone();
    if tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no ALGO transactions to sign!");
        Ok(state)
    } else {
        tx_infos
            .to_algo_signed_txs(
                &state.algo_db_utils.get_algo_fee()?,
                state.get_eth_submission_material()?.get_algo_first_valid_round()?,
                &state.algo_db_utils.get_genesis_hash()?,
                &state.algo_db_utils.get_redeem_address()?,
                &state.algo_db_utils.get_algo_private_key()?,
            )
            .and_then(|signed_txs| {
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_algo_signed_txs(&signed_txs)
            })
    }
}

// TODO Test
