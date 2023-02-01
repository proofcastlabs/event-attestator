use crate::{
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_constants::ZERO_ETH_VALUE,
        eth_contracts::erc777_token::encode_erc777_mint_fxn_maybe_with_data,
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::{EthTransaction, EthTransactions},
        },
        eth_database_utils::EthDbUtilsExt,
    },
    eos_on_int::eos::int_tx_info::EosOnIntIntTxInfos,
    metadata::ToMetadata,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

impl EosOnIntIntTxInfos {
    pub fn to_eth_signed_txs(
        &self,
        eth_account_nonce: u64,
        chain_id: &EthChainId,
        gas_price: u64,
        eth_private_key: &EthPrivateKey,
    ) -> Result<EthTransactions> {
        info!("✔ Getting INT signed transactions from `EosOnIntIntTxInfos`...");
        Ok(EthTransactions::new(
            self.iter()
                .enumerate()
                .map(|(i, tx_info)| {
                    info!("✔ Signing INT tx for info: {:?}", tx_info);
                    let operator_data = None;
                    EthTransaction::new_unsigned(
                        encode_erc777_mint_fxn_maybe_with_data(
                            // NOTE: This is destined for the interim chain, so we send the tokens
                            // to the router address. The true destination address is encoded in
                            // the metadata above.
                            &tx_info.router_address,
                            &tx_info.amount,
                            // NOTE: We're going to the interim chain so we always encode to metadata!
                            Some(tx_info.to_metadata_bytes()?),
                            operator_data,
                        )?,
                        eth_account_nonce + i as u64,
                        ZERO_ETH_VALUE,
                        tx_info.int_token_address,
                        chain_id,
                        chain_id.get_erc777_mint_with_data_gas_limit(),
                        gas_price,
                    )
                    .sign(eth_private_key)
                })
                .collect::<Result<Vec<EthTransaction>>>()?,
        ))
    }
}

pub fn maybe_sign_int_txs_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    let tx_infos = state.eos_on_int_int_tx_infos.clone();
    if tx_infos.is_empty() {
        info!("✔ No tx infos in state ∴ no INT transactions to sign!");
        Ok(state)
    } else {
        tx_infos
            .to_eth_signed_txs(
                state.eth_db_utils.get_eth_account_nonce_from_db()?,
                &state.eth_db_utils.get_eth_chain_id_from_db()?,
                state.eth_db_utils.get_eth_gas_price_from_db()?,
                &state.eth_db_utils.get_eth_private_key_from_db()?,
            )
            .and_then(|signed_txs| {
                debug!("✔ Signed transactions: {:?}", signed_txs);
                state.add_eth_signed_txs(signed_txs)
            })
    }
}
