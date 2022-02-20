use crate::{
    btc_on_int::btc::int_tx_info::BtcOnIntIntTxInfo,
    chains::{
        btc::{btc_chain_id::BtcChainId, btc_metadata::ToMetadata, btc_state::BtcState},
        eth::{
            eth_constants::MAX_BYTES_FOR_ETH_USER_DATA,
            eth_crypto::eth_transaction::{get_signed_minting_tx, EthTransaction, EthTransactions},
            eth_database_utils::EthDbUtilsExt,
            eth_types::EthSigningParams,
        },
    },
    metadata::metadata_protocol_id::MetadataProtocolId,
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_int_signed_txs(
    signing_params: &EthSigningParams,
    tx_infos: &[BtcOnIntIntTxInfo],
    btc_chain_id: &BtcChainId,
) -> Result<EthTransactions> {
    trace!("✔ Getting INT signed transactions...");
    Ok(EthTransactions::new(
        tx_infos
            .iter()
            .enumerate()
            .map(|(i, tx_info)| {
                info!(
                    "✔ Signing INT tx for host amount: {}, to destination address: {}",
                    tx_info.host_token_amount, tx_info.destination_address,
                );
                get_signed_minting_tx(
                    &tx_info.host_token_amount,
                    signing_params.eth_account_nonce + i as u64,
                    &signing_params.chain_id,
                    signing_params.smart_contract_address,
                    signing_params.gas_price,
                    &tx_info.router_address,
                    &signing_params.eth_private_key,
                    tx_info.maybe_to_metadata_bytes(
                        btc_chain_id,
                        MAX_BYTES_FOR_ETH_USER_DATA,
                        &MetadataProtocolId::Ethereum,
                    )?,
                    None,
                )
            })
            .collect::<Result<Vec<EthTransaction>>>()?,
    ))
}

pub fn maybe_sign_canon_block_txs<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    let tx_infos = state
        .btc_db_utils
        .get_btc_canon_block_from_db()?
        .get_btc_on_int_int_tx_infos();
    if tx_infos.is_empty() {
        info!("✔ No transactions to sign in canon block ∴ not signing anything!");
        Ok(state)
    } else {
        info!("✔ Signing INT txs from BTC canon block...");
        get_int_signed_txs(
            &state.eth_db_utils.get_signing_params_from_db()?,
            &tx_infos,
            &state.btc_db_utils.get_btc_chain_id_from_db()?,
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
