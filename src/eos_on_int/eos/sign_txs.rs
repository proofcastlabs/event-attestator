use crate::{
    chains::{
        eos::eos_state::EosState,
        eth::{
            eth_chain_id::EthChainId,
            eth_constants::ZERO_ETH_VALUE,
            eth_contracts::erc777::encode_erc777_mint_with_no_data_fxn,
            eth_crypto::{
                eth_private_key::EthPrivateKey,
                eth_transaction::{EthTransaction, EthTransactions},
            },
            eth_database_utils::EthDbUtilsExt,
        },
    },
    eos_on_int::eos::int_tx_info::EosOnIntIntTxInfos,
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
                    EthTransaction::new_unsigned(
                        encode_erc777_mint_with_no_data_fxn(&tx_info.destination_address, &tx_info.token_amount)?,
                        eth_account_nonce + i as u64,
                        ZERO_ETH_VALUE,
                        tx_info.eth_token_address,
                        chain_id,
                        chain_id.get_erc777_mint_with_no_data_gas_limit(),
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
                #[cfg(feature = "debug")]
                {
                    debug!("✔ Signed transactions: {:?}", signed_txs);
                }
                state.add_eth_signed_txs(signed_txs)
            })
    }
}
