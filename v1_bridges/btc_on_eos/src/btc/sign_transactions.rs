use common::{
    chains::btc::{btc_chain_id::BtcChainId, btc_metadata::ToMetadata},
    metadata::metadata_protocol_id::MetadataProtocolId,
    state::BtcState,
    traits::{DatabaseInterface, Serdable},
    types::Result,
    EosChainId,
};
use common_eos::{
    get_eos_tx_expiration_timestamp_with_offset,
    get_signed_eos_ptoken_issue_tx,
    EosDbUtils,
    EosPrivateKey,
    EosSignedTransaction,
    EosSignedTransactions,
    MAX_BYTES_FOR_EOS_USER_DATA,
};

use crate::btc::eos_tx_info::BtcOnEosEosTxInfos;

pub fn get_signed_eos_ptoken_issue_txs(
    ref_block_num: u16,
    ref_block_prefix: u32,
    chain_id: &EosChainId,
    pk: &EosPrivateKey,
    account: &str,
    eos_tx_infos: &BtcOnEosEosTxInfos,
    btc_chain_id: &BtcChainId,
) -> Result<EosSignedTransactions> {
    info!("✔ Signing {} txs...", eos_tx_infos.len());
    Ok(EosSignedTransactions::new(
        eos_tx_infos
            .iter()
            .enumerate()
            .map(|(i, params)| {
                get_signed_eos_ptoken_issue_tx(
                    ref_block_num,
                    ref_block_prefix,
                    &params.destination_address,
                    &params.amount,
                    chain_id,
                    pk,
                    account,
                    get_eos_tx_expiration_timestamp_with_offset(i as u32)?,
                    params.maybe_to_metadata_bytes(
                        btc_chain_id,
                        MAX_BYTES_FOR_EOS_USER_DATA,
                        &MetadataProtocolId::Eos,
                    )?,
                )
            })
            .collect::<Result<Vec<EosSignedTransaction>>>()?,
    ))
}

pub fn maybe_sign_canon_block_txs_and_add_to_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe signing EOS txs...");
    let eos_db_utils = EosDbUtils::new(state.db);
    get_signed_eos_ptoken_issue_txs(
        state.get_eos_ref_block_num()?,
        state.get_eos_ref_block_prefix()?,
        &eos_db_utils.get_eos_chain_id_from_db()?,
        &EosPrivateKey::get_from_db(state.db)?,
        &eos_db_utils.get_eos_account_name_string_from_db()?,
        &BtcOnEosEosTxInfos::from_bytes(&state.btc_db_utils.get_btc_canon_block_from_db()?.get_tx_info_bytes())?,
        &state.btc_db_utils.get_btc_chain_id_from_db()?,
    )
    .and_then(|eos_signed_txs| eos_signed_txs.to_bytes())
    .map(|bytes| state.add_eos_signed_txs(bytes))
}
